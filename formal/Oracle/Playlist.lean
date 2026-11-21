import Mpv.Common.Playlist

/-!
Runs the playlist model on a script of operations, one per line on stdin, and
prints the result and the playlist state after each. `fidelity/` feeds the same
script to mpv's real `playlist.c` and compares the output.
-/

open Mpv.Common.Playlist

namespace Oracle.Playlist

def empty : Playlist :=
  { entries := [], current := none, currentWasReplaced := false,
    playlistCompleted := false, playlistStarted := false, idAlloc := 0 }

/-- A freshly allocated entry that is in no playlist. -/
def detached : Entry := { id := 0, plIndex := 0, attached := false, removed := false }

def bit (b : Bool) : String := if b then "1" else "0"

def showEntry : Option Entry → String
  | some e => toString e.id
  | none => "none"

def showState (pl : Playlist) : String :=
  let ids := ",".intercalate (pl.entries.map (toString ·.id))
  let idx := ",".intercalate (pl.entries.map (toString ·.plIndex))
  let cur := match pl.current with
    | some id => toString id
    | none => "none"
  s!"ids={ids} idx={idx} cur={cur} r={bit pl.currentWasReplaced} " ++
    s!"c={bit pl.playlistCompleted} s={bit pl.playlistStarted} alloc={pl.idAlloc}"

/-- Script index to entry: `-1` is NULL, `-2` is the detached entry. -/
def entryArg (pl : Playlist) (i : Int) : Option Entry :=
  if i = -2 then some detached else if i < 0 then none else pl.entries[i.toNat]?

/-- Returns the state to print, the result, and the state to continue from. -/
def step (pl : Playlist) (op : String) (args : List Int) : Playlist × String × Playlist :=
  let query (result : String) := (pl, result, pl)
  let update (next : Playlist) := (next, "ok", next)
  match op, args with
  | "reset", [] => update empty
  | "insert", [a] => update (insertAt pl detached (entryArg pl a))
  | "remove", [i] =>
    match entryArg pl i with
    | some e => update (remove pl e).1
    | none => query "bad"
  | "setcur", [i] => update { pl with current := (entryArg pl i).map (·.id) }
  | "flags", [r, c, s] =>
    update { pl with currentWasReplaced := r ≠ 0, playlistCompleted := c ≠ 0,
                     playlistStarted := s ≠ 0 }
  | "count", [] => query (toString (entryCount pl))
  | "fromidx", [i] => query (showEntry (entryFromIndex pl i))
  | "toidx", [i] => query (toString (entryToIndex (entryArg pl i)))
  | "first", [] => query (showEntry (getFirst pl))
  | "last", [] => query (showEntry (getLast pl))
  | "rel", [i, d] =>
    match entryArg pl i with
    | some e => query (showEntry (entryGetRel pl e d))
    | none => query "bad"
  | "next", [d] => query (showEntry (getNext pl d))
  | "reindex", [i, v, s, e] =>
    let corrupted := if i < 0 then pl else
      { pl with entries := pl.entries.modify i.toNat fun x => { x with plIndex := v } }
    let shown := updateIndexes corrupted s e
    (shown, "ok", updateIndexes shown 0 (-1))
  | _, _ => query "bad"

partial def loop (stdin : IO.FS.Stream) (pl : Playlist) : IO Unit := do
  let line ← stdin.getLine
  if line.isEmpty then return
  match line.trimAscii.toString.splitOn " " with
  | op :: args =>
    let (shown, result, next) := step pl op (args.filterMap String.toInt?)
    IO.println s!"{result} | {showState shown}"
    loop stdin next
  | [] => loop stdin pl

end Oracle.Playlist

def main : IO Unit := do
  Oracle.Playlist.loop (← IO.getStdin) Oracle.Playlist.empty
