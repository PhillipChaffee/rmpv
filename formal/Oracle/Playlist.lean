import Mpv.Common.Playlist

/-!
Runs the playlist model on a script of operations, one per line on stdin, and
prints the result and the playlist state after each. `fidelity/` feeds the same
script to mpv's real `playlist.c` and compares the output.
-/

open Mpv.Common.Playlist

namespace Oracle.Playlist

/-! ## Stand-ins for callees

`playlist_entry_new` calls two functions from files the model does not cover
yet. These reproduce them only for the filenames the generator emits: ASCII
URLs, `-`, and absolute paths without a `..` component. -/

def isProtocolChar (c : Char) : Bool := c.isAlphanum || c == '.' || c == '-' || c == '+'

/-- `mp_split_proto`: the protocol and the part after `://`. -/
def splitProto (path : String) : Option (String × String) :=
  match path.splitOn "://" with
  | proto :: rest :: more =>
    match proto.toList with
    | first :: others =>
      if first.isAlpha && others.all isProtocolChar then
        some (proto, "://".intercalate (rest :: more))
      else none
    | [] => none
  | _ => none

def hexDigit (c : Char) : Option Nat :=
  if c.isDigit then some (c.toNat - '0'.toNat)
  else if 'a' ≤ c ∧ c ≤ 'f' then some (c.toNat - 'a'.toNat + 10)
  else if 'A' ≤ c ∧ c ≤ 'F' then some (c.toNat - 'A'.toNat + 10)
  else none

/-- `mp_url_unescape_inplace`. -/
def unescape : List Char → List Char
  | '%' :: a :: b :: rest =>
    match hexDigit a, hexDigit b with
    | some high, some low => Char.ofNat (16 * high + low) :: unescape rest
    | _, _ => '%' :: a :: b :: unescape rest
  | c :: rest => c :: unescape rest
  | [] => []

/-- `mp_file_url_to_filename`. -/
def fileUrlToFilename (url : String) : Option String :=
  match splitProto url with
  | some (proto, rest) =>
    if proto.toLower = "file" then some (String.ofList (unescape rest.toList)) else none
  | none => none

/-- `mp_normalize_path` on an absolute path, a URL, or `-`. -/
def normalizePath (path : String) : String :=
  if path.isEmpty || (splitProto path).isSome || path = "-" then path else
  let parts := (path.splitOn "/").filter fun part => part ≠ "" ∧ part ≠ "."
  if parts.isEmpty then "/" else String.join (parts.map ("/" ++ ·))

def newEntry (filename : String) : Entry := entryNew fileUrlToFilename normalizePath filename

/-! ## Script state -/

/-- The harness keeps one extra pointer to an entry it took a reference on. -/
inductive Held where
  | none
  /-- Still in the playlist, found by `id`. -/
  | listed (id : Nat)
  /-- Removed from the playlist but not freed. -/
  | loose (e : Entry)

structure World where
  pl : Playlist
  held : Held

def emptyPlaylist : Playlist :=
  { entries := [], current := none, currentWasReplaced := false,
    playlistCompleted := false, playlistStarted := false, playlistDir := none, idAlloc := 0 }

def empty : World := { pl := emptyPlaylist, held := .none }

/-- A freshly allocated entry that is in no playlist. -/
def detached : Entry := newEntry "/detached"

def heldEntry (w : World) : Option Entry :=
  match w.held with
  | .none => none
  | .listed id => w.pl.entries.find? (·.id = id)
  | .loose e => some e

def bit (b : Bool) : String := if b then "1" else "0"

def showEntry : Option Entry → String
  | some e => toString e.id
  | none => "none"

def showFields (e : Entry) : String :=
  let params := ",".intercalate (e.params.map fun p => s!"{p.1}={p.2}")
  s!"{e.id}:{e.plIndex}:a{bit e.attached}:x{bit e.removed}:r{e.reserved}:f{e.streamFlags}" ++
    s!":o{e.originalIndex}:{e.filename}:{e.playlistPath.getD "~"}:{params}"

def showState (w : World) : String :=
  let pl := w.pl
  let held := match heldEntry w with
    | some e => showFields e
    | none => "none"
  let current := match pl.current with
    | some id => toString id
    | none => "none"
  s!"cur={current} " ++
    s!"r={bit pl.currentWasReplaced} c={bit pl.playlistCompleted} " ++
    s!"s={bit pl.playlistStarted} dir={pl.playlistDir.getD "~"} alloc={pl.idAlloc} " ++
    s!"held={held} entries={";".intercalate (pl.entries.map showFields)}"

/-- Script index to entry: `-1` is NULL, `-2` the detached entry, `-3` the held one. -/
def entryArg (w : World) (i : Int) : Option Entry :=
  if i = -3 then heldEntry w
  else if i = -2 then some detached
  else if i < 0 then none
  else w.pl.entries[i.toNat]?

/-- The held entry leaves the playlist when it is among the removed survivors. -/
def afterRemoval (w : World) (pl : Playlist) (survivors : List Entry) : World :=
  let held := match w.held with
    | .listed id =>
      match survivors.find? (·.id = id) with
      | some e => .loose e
      | none => .listed id
    | other => other
  { pl, held }

def pairs : List String → List (String × String)
  | name :: value :: rest => (name, value) :: pairs rest
  | _ => []

/-- Returns the state to print, the result, and the state to continue from. -/
def step (w : World) (op : String) (words : List String) : World × String × World :=
  let pl := w.pl
  let query (result : String) := (w, result, w)
  let update (next : Playlist) := ({ w with pl := next }, "ok", { w with pl := next })
  let removal (count : Nat) (result : Playlist × List Entry) :=
    let next := afterRemoval w result.1 result.2
    (next, s!"freed={count - result.2.length}", next)
  let modify (index : String) (change : Entry → Entry) :=
    update { pl with entries := pl.entries.modify (index.toInt?.getD 0).toNat change }
  let bump (e : Entry) := { e with reserved := e.reserved + 1 }
  let args := words.filterMap String.toInt?
  match op, words with
  | "reset", [] => (empty, "ok", empty)
  | "new", [file] => query (showFields (newEntry file))
  | "insert", [a, file] => update (insertAt pl (newEntry file) (entryArg w (a.toInt?.getD (-1))))
  | "append", [file] => update (appendFile fileUrlToFilename normalizePath pl file)
  | "populate", [path] => update (populatePlaylistPath pl path)
  | "addparam", [i, name, value] =>
    modify i (entryAddParam · name value)
  | "addparams", i :: _ :: rest =>
    modify i (entryAddParams · (pairs rest))
  | "setparams", _ :: rest => update (setParams pl (pairs rest))
  | _, _ =>
  match op, args with
  | "remove", [i] =>
    match entryArg w i with
    | some e =>
      let removed := remove pl e
      removal 1 (removed.1, removed.2.toList)
    | none => query "bad"
  | "clear", [] => removal pl.entries.length (clear pl)
  | "clearx", [] =>
    let result := clearExceptCurrent pl
    removal (pl.entries.length - result.1.entries.length) result
  | "move", [i, a] =>
    match entryArg w i with
    | some e => update (move pl e (entryArg w a))
    | none => query "bad"
  | "setcur", [i] => update { pl with current := (entryArg w i).map (·.id) }
  | "flags", [r, c, s] =>
    update { pl with currentWasReplaced := r ≠ 0, playlistCompleted := c ≠ 0,
                     playlistStarted := s ≠ 0 }
  | "setdir", [k] => update { pl with playlistDir := if k ≠ 0 then some "/dir" else none }
  | "streamflags", [flags] => update (setStreamFlags pl flags)
  | "setcurrent", [] => update (setCurrent pl)
  | "hold", [i] =>
    match pl.entries[i.toNat]? with
    | some e =>
      let next := { pl := { pl with entries := pl.entries.modify i.toNat bump },
                    held := .listed e.id }
      (next, "ok", next)
    | none => query "bad"
  | "ref", [] =>
    let next := match w.held with
      | .none => w
      | .listed id =>
        { w with pl := { pl with entries := pl.entries.map fun e =>
            if e.id = id then bump e else e } }
      | .loose e => { w with held := .loose (bump e) }
    (next, "ok", next)
  | "unref", [] =>
    match w.held with
    | .none => query "bad"
    | .loose e =>
      let next := match entryUnref e with
        | some e' => { w with held := .loose e' }
        | none => { w with held := .none }
      (next, s!"freed={bit (entryUnref e).isNone}", next)
    | .listed id =>
      let unref (e : Entry) := if e.id = id then (entryUnref e).getD e else e
      let entries := pl.entries.map unref
      -- With no reference left the playlist is the only owner again.
      let stillHeld := entries.any fun e => e.id = id ∧ e.reserved > 0
      let next := { pl := { pl with entries }, held := if stillHeld then .listed id else .none }
      (next, "freed=0", next)
  | "count", [] => query (toString (entryCount pl))
  | "fromidx", [i] => query (showEntry (entryFromIndex pl i))
  | "toidx", [i] => query (toString (entryToIndex (entryArg w i)))
  | "first", [] => query (showEntry (getFirst pl))
  | "last", [] => query (showEntry (getLast pl))
  | "rel", [i, d] =>
    match entryArg w i with
    | some e => query (showEntry (entryGetRel pl e d))
    | none => query "bad"
  | "next", [d] => query (showEntry (getNext pl d))
  | "reindex", [i, v, s, e] =>
    let corrupted := if i < 0 then pl else
      { pl with entries := pl.entries.modify i.toNat fun x => { x with plIndex := v } }
    let shown := updateIndexes corrupted s e
    ({ w with pl := shown }, "ok", { w with pl := updateIndexes shown 0 (-1) })
  | _, _ => query "bad"

partial def loop (stdin : IO.FS.Stream) (w : World) : IO Unit := do
  let line ← stdin.getLine
  if line.isEmpty then return
  match line.trimAscii.toString.splitOn " " with
  | op :: words =>
    let (shown, result, next) := step w op words
    IO.println s!"{result} | {showState shown}"
    loop stdin next
  | [] => loop stdin w

end Oracle.Playlist

def main : IO Unit := do
  Oracle.Playlist.loop (← IO.getStdin) Oracle.Playlist.empty
