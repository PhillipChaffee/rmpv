import Mpv.Player.Audio
import Oracle.AvSync

/-!
Runs the audio model on a script of operations, one per line on stdin, and
prints the result and the state after each. `fidelity/` feeds the same script to
mpv's real `audio.c` and compares the output.
-/

open Mpv.Player.Audio
open Oracle.Script
open Oracle.AvSync (Clock)

namespace Oracle.Audio

/-- The state the script builds. -/
structure World where
  clock : Clock
  gain : Gain

def empty : World :=
  { clock := Clock.empty,
    gain := { softvolVolume := 0, softvolGain := 0, softvolMute := false, rgainMode := 0,
              rgainPreamp := 0, rgainClip := false, rgainFallback := 0, track := none } }

def showState (w : World) : String :=
  let g := w.gain
  let track := match g.track with
    | none => "none"
    | some none => "norg"
    | some (some rg) =>
      ",".intercalate ([rg.trackGain, rg.trackPeak, rg.albumGain, rg.albumPeak].map showFloat32)
  s!"{w.clock.show} vol={showFloat32 g.softvolVolume} sg={showFloat32 g.softvolGain} \
     mute={bit g.softvolMute} mode={g.rgainMode} pre={showFloat32 g.rgainPreamp} \
     clip={bit g.rgainClip} fb={showFloat32 g.rgainFallback} tr={track}"

/-- One script operation: its printed result and the state after it. -/
def step (w : World) (op : String) (args : List String) : String × World :=
  let g := w.gain
  let set (gain : Gain) : String × World := ("ok", { w with gain })
  match w.clock.set op args with
  | some clock => ("ok", { w with clock })
  | none =>
  match op, args with
  | "reset", [] => ("ok", empty)
  | "vol", [volume, gain, mute] =>
    set { g with softvolVolume := float32 volume, softvolGain := float32 gain,
                 softvolMute := int mute ≠ 0 }
  | "rgopts", [mode, preamp, clip, fallback] =>
    set { g with rgainMode := int mode, rgainPreamp := float32 preamp, rgainClip := int clip ≠ 0,
                 rgainFallback := float32 fallback }
  | "notrack", [] => set { g with track := none }
  | "norg", [] => set { g with track := some none }
  | "rg", [trackGain, trackPeak, albumGain, albumPeak] =>
    set { g with track := some (some { trackGain := float32 trackGain,
                                       trackPeak := float32 trackPeak,
                                       albumGain := float32 albumGain,
                                       albumPeak := float32 albumPeak }) }
  | "nopts", [] => (showFloat Mpv.Player.Playloop.nopts, w)
  | "dbgain", [db] => (showFloat (dbGain (float db)), w)
  | "replaygain", [] => (showFloat32 (computeReplaygain g), w)
  | "gain", [] => (showFloat32 (audioGetGain g), w)
  | "written", [] => (showFloat (writtenAudioPts w.clock.ctx), w)
  | "playing", [] => (showFloat (playingAudioPts w.clock.aoDelay w.clock.ctx), w)
  | _, _ => ("bad", w)

end Oracle.Audio

def main : IO Unit := do
  Oracle.Script.loop Oracle.Audio.step Oracle.Audio.showState (← IO.getStdin) Oracle.Audio.empty
