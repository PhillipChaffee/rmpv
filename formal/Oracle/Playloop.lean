import Mpv.Player.Playloop
import Oracle.Script

/-!
Runs the playloop model on a script of operations, one per line on stdin, and
prints the result and the state after each. `fidelity/` feeds the same script to
mpv's real `playloop.c` and compares the output. A float crosses the script and
the output as the 16 hex digits of its IEEE-754 bit pattern, so both sides start
from the identical value and the comparison is exact.
-/

open Mpv.Player.Playloop
open Oracle.Script

namespace Oracle.Playloop

/-- The state the script builds: the context, and what the stand-ins for
`get_play_start_pts` and `get_play_end_pts` return. -/
structure World where
  ctx : Ctx
  rangeStart : Float
  rangeEnd : Float

def empty : World :=
  { ctx := { demuxer := none, rebaseStartTime := false, playFrames := 0, playbackPts := 0,
             playDir := 0, lastSeekPts := 0, maxFrames := 0, chapters := #[],
             lastChapterSeek := 0, lastChapterFlag := false, cacheBuffer := 0 },
    rangeStart := 0, rangeEnd := 0 }

def showState (w : World) : String :=
  let c := w.ctx
  let demuxer := match c.demuxer with
    | some d => s!"{showFloat d.duration},{showFloat d.startTime},{d.filesize},{d.filepos}"
    | none => "none"
  let chapters := ",".intercalate (c.chapters.toList.map showFloat)
  s!"d={demuxer} rb={bit c.rebaseStartTime} pf={c.playFrames} pts={showFloat c.playbackPts} \
     dir={c.playDir} seek={showFloat c.lastSeekPts} mf={c.maxFrames} cs={c.lastChapterSeek} \
     cf={bit c.lastChapterFlag} cb={c.cacheBuffer} rs={showFloat w.rangeStart} \
     re={showFloat w.rangeEnd} ch={chapters}"

/-- One script operation: its printed result and the state after it. -/
def step (w : World) (op : String) (args : List String) : String × World :=
  let c := w.ctx
  let set (ctx : Ctx) : String × World := ("ok", { w with ctx })
  let time (x : Float) : String × World := (showFloat x, w)
  let number (n : Int) : String × World := (toString n, w)
  match op, args with
  | "reset", [] => ("ok", empty)
  | "nodemux", [] => set { c with demuxer := none }
  | "demux", [duration, start, size, pos] =>
    set { c with demuxer := some { duration := float duration, startTime := float start,
                                   filesize := int size, filepos := int pos } }
  | "opts", [rebase, frames] =>
    set { c with rebaseStartTime := int rebase ≠ 0, playFrames := int frames }
  | "time", [pts, dir, seek, frames] =>
    set { c with playbackPts := float pts, playDir := int dir, lastSeekPts := float seek,
                 maxFrames := int frames }
  | "chapters", _ :: times => set { c with chapters := (times.map float).toArray }
  | "chapseek", [seek, flag] =>
    set { c with lastChapterSeek := int seek, lastChapterFlag := int flag ≠ 0 }
  | "cache", [n] => set { c with cacheBuffer := int n }
  | "range", [start, stop] => ("ok", { w with rangeStart := float start, rangeEnd := float stop })
  | "nopts", [] => time nopts
  | "length", [] => time (getTimeLength c)
  | "start", [dir] => time (getStartTime c (int dir))
  | "current", [] => time (getCurrentTime c)
  | "playback", [] => time (getPlaybackTime c)
  | "ratio", [useRange] =>
    time (getCurrentPosRatio (fun _ => w.rangeStart) (fun _ => w.rangeEnd) c (int useRange ≠ 0))
  | "chapter", [] => number (getCurrentChapter c)
  | "chapstart", [n] => time (chapterStartTime c (int n))
  | "chapcount", [] => number (getChapterCount c)
  | "cachepct", [] => number (getCacheBufferingPercentage c)
  | _, _ => ("bad", w)

end Oracle.Playloop

def main : IO Unit := do
  Oracle.Script.loop Oracle.Playloop.step Oracle.Playloop.showState (← IO.getStdin)
    Oracle.Playloop.empty
