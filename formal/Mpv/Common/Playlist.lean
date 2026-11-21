/-!
Model of `common/playlist.c`.

A C `struct playlist_entry *` is modelled as an `Entry` value. A pointer is
"live" in a playlist when the list holds that exact value at `plIndex`.
C `int` is modelled as the unbounded `Int`.
-/

namespace Mpv.Common.Playlist

structure Entry where
  id : Nat
  /-- C `pl_index`; `-1` once detached. -/
  plIndex : Int
  /-- C `e->pl != NULL`. -/
  attached : Bool
  removed : Bool
  filename : String
  playlistPath : Option String
  /-- C `params[0 .. num_params)` as name and value pairs. -/
  params : List (String × String)
  originalIndex : Int
  /-- Extra references; the entry is freed when this drops below zero. -/
  reserved : Int
  streamFlags : Int
  deriving Repr, DecidableEq

structure Playlist where
  entries : List Entry
  /-- `id` of the C `current` entry. -/
  current : Option Nat
  currentWasReplaced : Bool
  playlistCompleted : Bool
  playlistStarted : Bool
  playlistDir : Option String
  idAlloc : Nat
  deriving Repr

/-- Every entry is attached and knows its own position. -/
def WF (pl : Playlist) : Prop :=
  ∀ (i : Nat) (h : i < pl.entries.length),
    pl.entries[i].plIndex = i ∧ pl.entries[i].attached = true

/-- `e` is a live pointer into `pl`. -/
def Live (pl : Playlist) (e : Entry) : Prop :=
  0 ≤ e.plIndex ∧ pl.entries[e.plIndex.toNat]? = some e

/-- C `STREAM_ORIGIN_DIRECT`. -/
def streamOriginDirect : Int := 4

/-- `fileUrlToFilename` and `normalizePath` stand for the callees
`mp_file_url_to_filename` and `mp_normalize_path`, which live in other files. -/
-- mpv-cover: common/playlist.c playlist_entry_new paths=all hash=bc763a57611e
def entryNew (fileUrlToFilename : String → Option String) (normalizePath : String → String)
    (filename : String) : Entry :=
  let path := match fileUrlToFilename filename with
    | some localFilename => localFilename
    | none => filename
  { id := 0, plIndex := 0, attached := false, removed := false,
    filename := normalizePath path, playlistPath := none, params := [],
    originalIndex := -1, reserved := 0, streamFlags := streamOriginDirect }

-- Path 1 only grows the array.
-- mpv-cover: common/playlist.c playlist_entry_add_param paths=all unobservable=1 hash=f16c5b49aeb1
def entryAddParam (e : Entry) (name value : String) : Entry :=
  { e with params := e.params ++ [(name, value)] }

-- mpv-cover: common/playlist.c playlist_entry_add_params paths=all hash=404214a643bf
def entryAddParams (e : Entry) (params : List (String × String)) : Entry :=
  params.foldl (fun e p => entryAddParam e p.1 p.2) e

-- mpv-cover: common/playlist.c playlist_set_params paths=all hash=018bb5c90520
def setParams (pl : Playlist) (params : List (String × String)) : Playlist :=
  { pl with entries := pl.entries.map fun e => entryAddParams { e with params := [] } params }

-- mpv-cover: common/playlist.c playlist_entry_count paths=all hash=f3806d9e16ea
def entryCount (pl : Playlist) : Int := pl.entries.length

-- mpv-cover: common/playlist.c playlist_entry_from_index paths=all hash=4a536e9f24c4
def entryFromIndex (pl : Playlist) (index : Int) : Option Entry :=
  if 0 ≤ index ∧ index < pl.entries.length then pl.entries[index.toNat]? else none

-- mpv-cover: common/playlist.c playlist_entry_to_index paths=all hash=dd3ad5325329
def entryToIndex (e : Option Entry) : Int :=
  match e with
  | none => -1
  | some e => if !e.attached then -1 else e.plIndex

-- mpv-cover: common/playlist.c playlist_update_indexes paths=all hash=913cb7a7e7c1
def updateIndexes (pl : Playlist) (start stop : Int) : Playlist :=
  let s := max start 0
  let e := if stop < 0 then (pl.entries.length : Int) else min stop pl.entries.length
  { pl with entries := pl.entries.mapIdx fun n x =>
      if s ≤ (n : Int) ∧ (n : Int) < e then { x with plIndex := n } else x }

-- Path 10 only grows the array.
-- mpv-cover: common/playlist.c playlist_insert_at paths=all unobservable=10 hash=4ec8fe004600
def insertAt (pl : Playlist) (add : Entry) (at_ : Option Entry) : Playlist :=
  let index : Int := match at_ with
    | some a => a.plIndex
    | none => pl.entries.length
  let add' := { add with attached := true, plIndex := index, id := pl.idAlloc + 1 }
  let pl' := { pl with entries := pl.entries.insertIdx index.toNat add', idAlloc := pl.idAlloc + 1 }
  updateIndexes pl' index pl'.entries.length

-- mpv-cover: common/playlist.c playlist_entry_get_rel paths=all hash=aae4ee6b4cb0
def entryGetRel (pl : Playlist) (e : Entry) (direction : Int) : Option Entry :=
  if !e.attached then none else entryFromIndex pl (e.plIndex + direction)

/-- `none` means the entry was freed. -/
-- mpv-cover: common/playlist.c playlist_entry_unref paths=all hash=db7fa2d6e029
def entryUnref (e : Entry) : Option Entry :=
  let e' := { e with reserved := e.reserved - 1 }
  if e'.reserved < 0 then none else some e'

/-- Returns the playlist and the removed entry, or `none` if removing it freed it. -/
-- mpv-cover: common/playlist.c playlist_remove paths=all hash=bc3bd697e2c4
def remove (pl : Playlist) (entry : Entry) : Playlist × Option Entry :=
  let pl1 := if pl.current = some entry.id then
      { pl with current := (entryGetRel pl entry 1).map (·.id), currentWasReplaced := true }
    else pl
  let pl2 := { pl1 with entries := pl1.entries.eraseIdx entry.plIndex.toNat }
  (updateIndexes pl2 entry.plIndex (-1),
   entryUnref { entry with attached := false, plIndex := -1, removed := true })

/-- The loop `for (n = num_entries - 1; n >= 0; n--)` that `playlist_clear` and
`playlist_clear_except_current` share, started at `n - 1`. Returns the playlist
and the removed entries that were not freed, in removal order. -/
def removeBackwards (keepCurrent : Bool) (pl : Playlist) : Nat → Playlist × List Entry
  | 0 => (pl, [])
  | n + 1 =>
    match pl.entries[n]? with
    | none => removeBackwards keepCurrent pl n
    | some e =>
      if keepCurrent ∧ pl.current = some e.id then removeBackwards keepCurrent pl n
      else
        let removed := remove pl e
        let rest := removeBackwards keepCurrent removed.1 n
        (rest.1, removed.2.toList ++ rest.2)

-- mpv-cover: common/playlist.c playlist_clear paths=all hash=7f28b9ff36c9
def clear (pl : Playlist) : Playlist × List Entry :=
  let removed := removeBackwards false pl pl.entries.length
  ({ removed.1 with currentWasReplaced := false, playlistCompleted := false,
                    playlistStarted := false, playlistDir := none }, removed.2)

-- mpv-cover: common/playlist.c playlist_clear_except_current paths=all hash=7ebcbffc32c2
def clearExceptCurrent (pl : Playlist) : Playlist × List Entry :=
  let removed := removeBackwards true pl pl.entries.length
  ({ removed.1 with playlistCompleted := false, playlistStarted := false }, removed.2)

-- Path 12 only grows the array.
-- mpv-cover: common/playlist.c playlist_move paths=all unobservable=12 hash=0ba4d28b8cd4
def move (pl : Playlist) (entry : Entry) (at_ : Option Entry) : Playlist :=
  if some entry = at_ then pl else
  let index : Int := match at_ with
    | some a => a.plIndex
    | none => pl.entries.length
  let inserted := pl.entries.insertIdx index.toNat entry
  let oldIndex := if entry.plIndex ≥ index then entry.plIndex + 1 else entry.plIndex
  let pl' := { pl with entries := inserted.eraseIdx oldIndex.toNat }
  updateIndexes pl' (min (index - 1) (oldIndex - 1)) (max (index + 1) (oldIndex + 1))

-- mpv-cover: common/playlist.c playlist_append_file paths=all hash=d74956982b0d
def appendFile (fileUrlToFilename : String → Option String) (normalizePath : String → String)
    (pl : Playlist) (filename : String) : Playlist :=
  insertAt pl (entryNew fileUrlToFilename normalizePath filename) none

-- mpv-cover: common/playlist.c playlist_populate_playlist_path paths=all hash=ba11cc8002dd
def populatePlaylistPath (pl : Playlist) (path : String) : Playlist :=
  { pl with entries := pl.entries.map fun e => { e with playlistPath := some path } }

-- mpv-cover: common/playlist.c playlist_set_stream_flags paths=all hash=c329abd605e3
def setStreamFlags (pl : Playlist) (flags : Int) : Playlist :=
  { pl with entries := pl.entries.map fun e => { e with streamFlags := flags } }

-- mpv-cover: common/playlist.c playlist_set_current paths=all hash=7798e361b4c5
def setCurrent (pl : Playlist) : Playlist :=
  match pl.playlistDir with
  | none => pl
  | some _ =>
    match pl.entries.find? fun e => e.playlistPath = some e.filename with
    | some e => { pl with current := some e.id }
    | none => pl

-- mpv-cover: common/playlist.c playlist_get_first paths=all hash=2d8dc5c29713
def getFirst (pl : Playlist) : Option Entry :=
  if pl.entries.length ≠ 0 then pl.entries[0]? else none

-- mpv-cover: common/playlist.c playlist_get_last paths=all hash=335505d65ea1
def getLast (pl : Playlist) : Option Entry :=
  if pl.entries.length ≠ 0 then pl.entries[pl.entries.length - 1]? else none

-- mpv-cover: common/playlist.c playlist_get_next paths=all hash=97ab285258b3
def getNext (pl : Playlist) (direction : Int) : Option Entry :=
  match pl.current with
  | none =>
    if pl.playlistCompleted ∧ direction < 0 then entryFromIndex pl (pl.entries.length - 1)
    else if !pl.playlistStarted ∧ direction > 0 then entryFromIndex pl 0
    else none
  | some c =>
    match pl.entries.find? (·.id = c) with
    | none => none
    | some cur =>
      if direction < 0 then entryGetRel pl cur (-1)
      else if pl.currentWasReplaced then some cur
      else entryGetRel pl cur 1

/-- Exchanges `l[i]` and `l[j]`. `playlist_shuffle` only passes indexes inside
the array, so the out-of-range case is never used. -/
def swap (l : List Entry) (i j : Nat) : List Entry :=
  match l[i]?, l[j]? with
  | some a, some b => (l.set i b).set j a
  | _, _ => l

/-- `rand k min max` stands for the result of the `k`-th call
`mp_rand_in_range32(&s, min, max)`, which lives in `misc/random.c`. -/
-- mpv-cover: common/playlist.c playlist_shuffle paths=all hash=1ac3af6e0b61
def shuffle (rand : Nat → Nat → Nat → Nat) (pl : Playlist) : Playlist :=
  let marked := pl.entries.mapIdx fun n e => { e with originalIndex := n }
  let swapped := (List.range (marked.length - 1)).foldl
    (fun l n => swap l n (rand n n marked.length)) marked
  updateIndexes { pl with entries := swapped } 0 (-1)

/-- C `CMP_INT`. -/
def cmpInt (a b : Int) : Int := if a = b then 0 else if a > b then 1 else -1

-- mpv-cover: common/playlist.c cmp_unshuffle paths=all hash=f4593622a6e1
def cmpUnshuffle (a b : Entry) : Int :=
  if a.originalIndex ≥ 0 ∧ a.originalIndex ≠ b.originalIndex then
    cmpInt a.originalIndex b.originalIndex
  else cmpInt a.plIndex b.plIndex

/-- `sort` stands for libc's `qsort`. C defines its result only when the
comparison is a consistent total order on the array. `cmpUnshuffle` is one
exactly when no entry with `original_index < 0` sits after an entry with
`original_index ≥ 0`. -/
-- mpv-cover: common/playlist.c playlist_unshuffle paths=all hash=3532f5551af0
def unshuffle (sort : (Entry → Entry → Int) → List Entry → List Entry) (pl : Playlist) :
    Playlist :=
  let sorted := if pl.entries.length ≠ 0 then sort cmpUnshuffle pl.entries else pl.entries
  updateIndexes { pl with entries := sorted } 0 (-1)

/-- The last loop of `playlist_get_first_in_same_playlist`: step back while the
previous entry has `path`. `fuel` bounds the steps; the playlist length is enough. -/
def firstWithPath (pl : Playlist) (path : String) : Nat → Entry → Entry
  | 0, entry => entry
  | fuel + 1, entry =>
    match entryGetRel pl entry (-1) with
    | some prev =>
      if prev.playlistPath = some path then firstWithPath pl path fuel prev else entry
    | none => entry

/-- `stripTrailingSeparator` stands for `mp_path_strip_trailing_separator` in
`misc/path_utils.c`. `pl` is the playlist `entry` is in. A NULL
`current_playlist_path` is `none`; C reads it as the empty string. -/
-- mpv-cover: common/playlist.c playlist_get_first_in_same_playlist paths=all hash=55564e6a9240
def getFirstInSamePlaylist (stripTrailingSeparator : String → String) (pl : Playlist)
    (entry : Option Entry) (currentPlaylistPath : Option String) : Option Entry :=
  match entry with
  | none => none
  | some e =>
    match e.playlistPath with
    | none => some e
    | some path =>
      if (currentPlaylistPath.getD "").startsWith (stripTrailingSeparator path ++ "/") then some e
      else some (firstWithPath pl path pl.entries.length e)

/-- The first loop of `playlist_get_first_in_next_playlist`: step in `direction`
while the entry has the current entry's playlist path. -/
def skipCurrentPlaylist (pl : Playlist) (currentPath : Option String) (direction : Int) :
    Nat → Option Entry → Option Entry
  | _, none => none
  | 0, some e => some e
  | fuel + 1, some e =>
    if e.playlistPath.isSome ∧ e.playlistPath = currentPath then
      skipCurrentPlaylist pl currentPath direction fuel (entryGetRel pl e direction)
    else some e

/-- C reads `pl->current->playlist_path` without checking `pl->current`. With
no current entry that is a NULL dereference, unless `playlist_get_next` returns
NULL, or `direction > 0` and the entry it returns has no playlist path. The
model describes mpv only outside that case. -/
-- mpv-cover: common/playlist.c playlist_get_first_in_next_playlist paths=all hash=37e5f7480a10
def getFirstInNextPlaylist (stripTrailingSeparator : String → String) (pl : Playlist)
    (direction : Int) : Option Entry :=
  match getNext pl direction with
  | none => none
  | some next =>
    let currentPath :=
      (pl.current.bind fun c => pl.entries.find? (·.id = c)).bind (·.playlistPath)
    let entry := skipCurrentPlaylist pl currentPath direction pl.entries.length (some next)
    if direction < 0 then getFirstInSamePlaylist stripTrailingSeparator pl entry currentPath
    else entry

/-- Returns the destination, the emptied source, and the C return value. C
leaves `source_pl->current` pointing at its old entry, which now has its new
`id`. Path 5 only grows the array. -/
-- mpv-cover: common/playlist.c playlist_transfer_entries_to paths=all unobservable=5 hash=9d2c4074f0ba
def transferEntriesTo (pl : Playlist) (dstIndex : Int) (source : Playlist) :
    Playlist × Playlist × Int :=
  let moved := source.entries.mapIdx fun n e =>
    { e with attached := true, plIndex := dstIndex + n, id := pl.idAlloc + n + 1 }
  let entries := pl.entries.take dstIndex.toNat ++ moved ++ pl.entries.drop dstIndex.toNat
  let inserted := { pl with entries, idAlloc := pl.idAlloc + moved.length }
  let reindexed := updateIndexes inserted (dstIndex + moved.length) (-1)
  let first : Int := match moved.head? with
    | some e => e.id
    | none => 0
  ({ reindexed with playlistCompleted := source.playlistCompleted,
                    playlistStarted := source.playlistStarted },
   { source with
     entries := [],
     current := source.current.bind fun c =>
       (source.entries.findIdx? (·.id = c)).map (pl.idAlloc + · + 1) },
   first)

-- mpv-cover: common/playlist.c playlist_transfer_entries paths=all hash=b657edce44bf
def transferEntries (pl source : Playlist) : Playlist × Playlist × Int :=
  let addAt : Int := match pl.current.bind fun c => pl.entries.find? (·.id = c) with
    | none => pl.entries.length
    | some current =>
      if pl.currentWasReplaced then current.plIndex + 1 + 1 else current.plIndex + 1
  transferEntriesTo pl addAt source

-- mpv-cover: common/playlist.c playlist_append_entries paths=all hash=999e328440cb
def appendEntries (pl source : Playlist) : Playlist × Playlist × Int :=
  transferEntriesTo pl pl.entries.length source

/-- C `talloc_zero(NULL, struct playlist)`. -/
def Playlist.zeroed : Playlist :=
  { entries := [], current := none, currentWasReplaced := false, playlistCompleted := false,
    playlistStarted := false, playlistDir := none, idAlloc := 0 }

/-- What `playlist_parse_file` reads from the demuxer `demux_open_url` returns. -/
structure Demuxer where
  playlist : Option Playlist
  filetype : Option String

/-- `getUserPath` stands for `mp_get_user_path` in `options/path.c`, and
`demuxOpenUrl path forceFormat streamFlags` for `demux_open_url` in
`demux/demux.c`; `none` is a NULL demuxer. The demuxer and its emptied playlist
are freed, so only the new playlist is returned. Paths 4 to 9 only choose which
message to log; the file type is read for nothing else. -/
-- mpv-cover: common/playlist.c playlist_parse_file paths=0,1,2,3 unobservable=4,5,6,7,8,9 hash=8d8c69321ea1
def parseFile (getUserPath : String → String)
    (demuxOpenUrl : String → String → Int → Option Demuxer) (file : String) : Option Playlist :=
  match demuxOpenUrl (getUserPath file) "playlist" streamOriginDirect with
  | none => none
  | some demuxer =>
    match demuxer.playlist with
    | none => none
    | some playlist =>
      some (transferEntries Playlist.zeroed (populatePlaylistPath playlist file)).1

/-! ## Theorems -/

theorem updateIndexes_length (pl : Playlist) (s e : Int) :
    (updateIndexes pl s e).entries.length = pl.entries.length := by
  simp [updateIndexes]

theorem updateIndexes_getElem (pl : Playlist) (s e : Int) (i : Nat)
    (h : i < (updateIndexes pl s e).entries.length) :
    (updateIndexes pl s e).entries[i] =
      if max s 0 ≤ (i : Int) ∧
          (i : Int) < (if e < 0 then (pl.entries.length : Int) else min e pl.entries.length)
      then { pl.entries[i]'(by simpa [updateIndexes] using h) with plIndex := i }
      else pl.entries[i]'(by simpa [updateIndexes] using h) := by
  simp [updateIndexes, List.getElem_mapIdx]

/-- `playlist_update_indexes` restores `WF` when every misplaced `pl_index` is
in its range. -/
theorem updateIndexes_wf_of (pl : Playlist) (s e : Int)
    (h : ∀ (k : Nat) (hk : k < pl.entries.length), pl.entries[k].attached = true ∧
      (pl.entries[k].plIndex = k ∨ (max s 0 ≤ (k : Int) ∧
        (k : Int) < (if e < 0 then (pl.entries.length : Int) else min e pl.entries.length)))) :
    WF (updateIndexes pl s e) := by
  intro k hk
  rw [updateIndexes_getElem]
  obtain ⟨hatt, hor⟩ := h k (by simpa [updateIndexes] using hk)
  by_cases hc : max s 0 ≤ (k : Int) ∧
      (k : Int) < (if e < 0 then (pl.entries.length : Int) else min e pl.entries.length)
  · rw [ite_eq_left_of_eq_true _ _ (eq_true hc)]
    exact ⟨rfl, hatt⟩
  · rw [ite_eq_right_of_eq_false _ _ (eq_false hc)]
    exact ⟨hor.resolve_right hc, hatt⟩

theorem live_lt {pl : Playlist} {e : Entry} (h : Live pl e) :
    e.plIndex.toNat < pl.entries.length :=
  (List.getElem?_eq_some_iff.mp h.2).1

theorem insertAt_count (pl : Playlist) (add : Entry) (at_ : Option Entry)
    (h : ∀ a, at_ = some a → Live pl a) :
    entryCount (insertAt pl add at_) = entryCount pl + 1 := by
  cases at_ with
  | none => simp [insertAt, entryCount, updateIndexes]
  | some a =>
    have hlt := live_lt (h a rfl)
    have h0 := (h a rfl).1
    simp [insertAt, entryCount, updateIndexes, List.length_insertIdx]
    split <;> omega

theorem remove_count (pl : Playlist) (entry : Entry) (h : Live pl entry) :
    entryCount (remove pl entry).1 = entryCount pl - 1 := by
  have hlt := live_lt h
  have hlen : ∀ l : List Entry, entry.plIndex.toNat < l.length →
      ((l.eraseIdx entry.plIndex.toNat).length : Int) = l.length - 1 := by
    intro l hl; rw [List.length_eraseIdx]; simp [hl]; omega
  unfold remove entryCount
  simp only [updateIndexes_length]
  split <;> exact hlen _ hlt

/-- `playlist_insert_at` keeps every `pl_index` equal to its position. -/
theorem insertAt_wf (pl : Playlist) (add : Entry) (at_ : Option Entry)
    (hwf : WF pl) (h : ∀ a, at_ = some a → Live pl a) : WF (insertAt pl add at_) := by
  have key : ∀ index : Int, 0 ≤ index → index.toNat ≤ pl.entries.length →
      WF (updateIndexes
        { pl with
          entries := pl.entries.insertIdx index.toNat
            { add with attached := true, plIndex := index, id := pl.idAlloc + 1 },
          idAlloc := pl.idAlloc + 1 }
        index ((pl.entries.insertIdx index.toNat
            { add with attached := true, plIndex := index, id := pl.idAlloc + 1 }).length)) := by
    intro index h0 hle i hi
    rw [updateIndexes_getElem]
    have hi' : i < pl.entries.length + 1 := by
      simpa [updateIndexes, List.length_insertIdx, hle] using hi
    simp only [List.getElem_insertIdx, List.length_insertIdx, hle]
    by_cases h1 : i < index.toNat
    · have := hwf i (by omega)
      simp [h1]; split <;> simp_all <;> omega
    · by_cases h2 : i = index.toNat
      · simp [h2]; split <;> simp_all <;> omega
      · have := hwf (i - 1) (by omega)
        simp [h1, h2]; split <;> simp_all <;> omega
  cases at_ with
  | none => simpa [insertAt] using key pl.entries.length (by omega) (by simp)
  | some a =>
    have hlt := live_lt (h a rfl)
    exact key a.plIndex (h a rfl).1 (by omega)

/-- `playlist_remove` keeps every `pl_index` equal to its position. -/
theorem remove_wf (pl : Playlist) (entry : Entry)
    (hwf : WF pl) (h : Live pl entry) : WF (remove pl entry).1 := by
  have hlt := live_lt h
  have h0 := h.1
  have key : ∀ pl1 : Playlist, pl1.entries = pl.entries →
      WF (updateIndexes { pl1 with entries := pl1.entries.eraseIdx entry.plIndex.toNat }
        entry.plIndex (-1)) := by
    intro pl1 he i hi
    rw [updateIndexes_getElem]
    have hi' : i < pl.entries.length - 1 := by
      simpa [updateIndexes, List.length_eraseIdx, he, hlt] using hi
    have hd : (pl.entries.eraseIdx entry.plIndex.toNat).length = pl.entries.length - 1 := by
      rw [List.length_eraseIdx]; simp [hlt]
    simp only [List.getElem_eraseIdx, he]
    by_cases h1 : i < entry.plIndex.toNat
    · have := hwf i (by omega)
      simp [h1]; split <;> simp_all <;> omega
    · have := hwf (i + 1) (by omega)
      simp [h1]; split <;> simp_all <;> omega
  unfold remove
  split
  · exact key _ rfl
  · exact key _ rfl

/-- `playlist_entry_to_index` inverts `playlist_entry_from_index`. -/
theorem toIndex_fromIndex (pl : Playlist) (i : Int) (e : Entry)
    (hwf : WF pl) (h : entryFromIndex pl i = some e) : entryToIndex (some e) = i := by
  unfold entryFromIndex at h
  split at h
  · rename_i hr
    obtain ⟨hl, he⟩ := List.getElem?_eq_some_iff.mp h
    have := hwf i.toNat hl
    rw [he] at this
    simp [entryToIndex, this.2, this.1]; omega
  · simp at h

/-- A fresh playlist starts at its first entry. -/
theorem getNext_unstarted (pl : Playlist) (hc : pl.current = none)
    (hs : pl.playlistStarted = false) : getNext pl 1 = getFirst pl := by
  simp [getNext, hc, hs, entryFromIndex, getFirst]
  cases pl.entries <;> simp

/-- `playlist_entry_unref` frees the entry exactly when no extra reference is left. -/
theorem entryUnref_eq_none (e : Entry) : entryUnref e = none ↔ e.reserved ≤ 0 := by
  simp only [entryUnref]
  split <;> simp_all <;> omega

/-- `playlist_update_indexes` changes nothing but `pl_index`. -/
theorem updateIndexes_map {β : Type} (pl : Playlist) (s e : Int) (f : Entry → β)
    (hf : ∀ (x : Entry) (i : Int), f { x with plIndex := i } = f x) :
    (updateIndexes pl s e).entries.map f = pl.entries.map f := by
  apply List.ext_getElem
  · simp [updateIndexes]
  · intro i h1 h2
    rw [List.getElem_map, List.getElem_map, updateIndexes_getElem]
    by_cases hc : max s 0 ≤ (i : Int) ∧
        (i : Int) < (if e < 0 then (pl.entries.length : Int) else min e pl.entries.length)
    · rw [ite_eq_left_of_eq_true _ _ (eq_true hc)]; exact hf _ _
    · rw [ite_eq_right_of_eq_false _ _ (eq_false hc)]

theorem remove_ids (pl : Playlist) (entry : Entry) :
    (remove pl entry).1.entries.map (·.id) =
      (pl.entries.map (·.id)).eraseIdx entry.plIndex.toNat := by
  unfold remove
  rw [updateIndexes_map _ _ _ _ (fun _ _ => rfl)]
  split <;> simp [List.eraseIdx_eq_take_drop_succ]

theorem remove_current (pl : Playlist) (entry : Entry) (h : pl.current ≠ some entry.id) :
    (remove pl entry).1.current = pl.current := by
  simp [remove, updateIndexes, h]

/-- A `WF` playlist holds `entries[n]` as a live pointer. -/
theorem live_getElem {pl : Playlist} (hwf : WF pl) {n : Nat} {e : Entry}
    (h : pl.entries[n]? = some e) : Live pl e ∧ e.plIndex.toNat = n := by
  obtain ⟨hl, he⟩ := List.getElem?_eq_some_iff.mp h
  have := (hwf n hl).1
  rw [he] at this
  refine ⟨⟨by omega, ?_⟩, by omega⟩
  rw [this]; simpa using h

theorem removeBackwards_wf (keep : Bool) (pl : Playlist) (n : Nat) (hwf : WF pl) :
    WF (removeBackwards keep pl n).1 := by
  induction n generalizing pl with
  | zero => exact hwf
  | succ n ih =>
    unfold removeBackwards
    split
    · exact ih pl hwf
    · rename_i e he
      split
      · exact ih pl hwf
      · exact ih _ (remove_wf pl e hwf (live_getElem hwf he).1)

theorem removeBackwards_all_length (pl : Playlist) (n : Nat) (hwf : WF pl)
    (hn : n ≤ pl.entries.length) :
    (removeBackwards false pl n).1.entries.length = pl.entries.length - n := by
  induction n generalizing pl with
  | zero => rfl
  | succ n ih =>
    have ⟨e, he⟩ : ∃ e, pl.entries[n]? = some e := ⟨pl.entries[n], by simp⟩
    have hlive := (live_getElem hwf he).1
    have hcount := remove_count pl e hlive
    simp only [entryCount] at hcount
    simp only [removeBackwards, he, Bool.false_eq_true, false_and, ↓reduceIte]
    rw [ih _ (remove_wf pl e hwf hlive) (by omega)]
    omega

/-- Removing from the back leaves no successor to become `current`. -/
theorem removeBackwards_all_current (pl : Playlist) (n : Nat) (hwf : WF pl)
    (hn : n = pl.entries.length)
    (hc : ∀ c, pl.current = some c → c ∈ pl.entries.map (·.id)) :
    (removeBackwards false pl n).1.current = none := by
  induction n generalizing pl with
  | zero =>
    cases hcur : pl.current with
    | none => simpa [removeBackwards] using hcur
    | some c =>
      have := hc c hcur
      have h0 : pl.entries = [] := List.eq_nil_of_length_eq_zero hn.symm
      simp [h0] at this
  | succ n ih =>
    have ⟨e, he⟩ : ∃ e, pl.entries[n]? = some e := ⟨pl.entries[n], by simp⟩
    obtain ⟨hlive, hidx⟩ := live_getElem hwf he
    have hcount := remove_count pl e hlive
    simp only [entryCount] at hcount
    simp only [removeBackwards, he, Bool.false_eq_true, false_and, ↓reduceIte]
    apply ih _ (remove_wf pl e hwf hlive) (by omega)
    intro c hcur
    rw [remove_ids, hidx]
    by_cases hce : pl.current = some e.id
    · have hrel : entryGetRel pl e 1 = none := by
        have := hlive.1
        simp [entryGetRel, entryFromIndex,
          (List.getElem?_eq_some_iff.mp he).2 ▸ (hwf n (by omega)).2]
        omega
      simp [remove, updateIndexes, hce, hrel] at hcur
    · rw [remove_current pl e hce] at hcur
      have hmem := hc c hcur
      obtain ⟨i, hi, hci⟩ := List.getElem_of_mem hmem
      rw [List.mem_eraseIdx_iff_getElem]
      refine ⟨i, hi, ?_, hci⟩
      intro hin
      subst hin
      have : e.id = c := by
        simp only [List.getElem_map] at hci
        rw [(List.getElem?_eq_some_iff.mp he).2] at hci
        exact hci
      exact hce (by rw [hcur, this])

/-- After `playlist_clear` there are no entries and no current entry. The
hypothesis on `current` is what makes `mp_assert(!pl->current)` hold. -/
theorem clear_empty (pl : Playlist) (hwf : WF pl)
    (hc : ∀ c, pl.current = some c → c ∈ pl.entries.map (·.id)) :
    (clear pl).1.entries = [] ∧ (clear pl).1.current = none := by
  constructor
  · apply List.eq_nil_of_length_eq_zero
    simpa [clear] using removeBackwards_all_length pl _ hwf (Nat.le_refl _)
  · simpa [clear] using removeBackwards_all_current pl _ hwf rfl hc

/-- The loop of `playlist_clear_except_current` keeps exactly the entries that
are `current`, and leaves `current` alone. -/
theorem removeBackwards_keep (pl : Playlist) (n : Nat) (hwf : WF pl)
    (hn : n ≤ pl.entries.length) :
    (removeBackwards true pl n).1.current = pl.current ∧
    (removeBackwards true pl n).1.entries.map (·.id) =
      ((pl.entries.map (·.id)).take n).filter (fun i => pl.current == some i) ++
        (pl.entries.map (·.id)).drop n := by
  induction n generalizing pl with
  | zero => simp [removeBackwards]
  | succ n ih =>
    have ⟨e, he⟩ : ∃ e, pl.entries[n]? = some e := ⟨pl.entries[n], by simp⟩
    obtain ⟨hlive, hidx⟩ := live_getElem hwf he
    have hid : (pl.entries.map (·.id))[n]? = some e.id := by simp [he]
    have hlen : n < (pl.entries.map (·.id)).length := by simp; omega
    have hdrop : (pl.entries.map (·.id)).drop n = e.id :: (pl.entries.map (·.id)).drop (n + 1) := by
      rw [List.drop_eq_getElem_cons hlen]
      congr 1
      exact Option.some.inj ((List.getElem?_eq_getElem hlen).symm.trans hid)
    by_cases hcur : pl.current = some e.id
    · have := ih pl hwf (by omega)
      simp only [removeBackwards, he, hcur, and_self, ↓reduceIte]
      rw [hcur] at this
      refine ⟨this.1, ?_⟩
      rw [this.2, List.take_add_one, hid, hdrop]
      simp
    · have hcount := remove_count pl e hlive
      simp only [entryCount] at hcount
      have := ih _ (remove_wf pl e hwf hlive) (by omega)
      rw [remove_current pl e hcur, remove_ids, hidx] at this
      simp only [removeBackwards, he, hcur, and_false, ↓reduceIte]
      refine ⟨this.1, ?_⟩
      have htake : ((pl.entries.map (·.id)).eraseIdx n).take n = (pl.entries.map (·.id)).take n := by
        grind
      have hdrop' : ((pl.entries.map (·.id)).eraseIdx n).drop n =
          (pl.entries.map (·.id)).drop (n + 1) := by
        grind
      rw [this.2, htake, hdrop', List.take_add_one, hid]
      simp [hcur]

theorem filter_eq_length_le_one (l : List Nat) (c : Option Nat) (h : l.Nodup) :
    (l.filter fun i => c == some i).length ≤ 1 := by
  induction l with
  | nil => simp
  | cons a l ih =>
    obtain ⟨ha, hl⟩ := List.nodup_cons.mp h
    by_cases hc : c = some a
    · have : l.filter (fun i => c == some i) = [] := by
        rw [List.filter_eq_nil_iff]
        intro x hx hcx
        have : a = x := by simpa [hc] using hcx
        exact ha (this ▸ hx)
      rw [List.filter_cons]
      simp [hc] at this ⊢
      exact this
    · have := ih hl
      simpa [List.filter_cons, hc] using this

/-- `playlist_clear_except_current` keeps the entries whose pointer is
`current`. Pointer identity is `id` in the model. -/
theorem clearExceptCurrent_ids (pl : Playlist) (hwf : WF pl) :
    (clearExceptCurrent pl).1.current = pl.current ∧
    (clearExceptCurrent pl).1.entries.map (·.id) =
      (pl.entries.map (·.id)).filter fun i => pl.current == some i := by
  have := removeBackwards_keep pl pl.entries.length hwf (Nat.le_refl _)
  have hlen : pl.entries.length = (pl.entries.map (·.id)).length := by simp
  simp only [clearExceptCurrent]
  rw [hlen, List.take_length, List.drop_length, List.append_nil, ← hlen] at this
  exact this

/-- `playlist_clear_except_current` preserves `WF` and leaves at most one
entry. In C the entries are distinct pointers; here that is distinct `id`s. -/
theorem clearExceptCurrent_wf (pl : Playlist) (hwf : WF pl)
    (hid : (pl.entries.map (·.id)).Nodup) :
    WF (clearExceptCurrent pl).1 ∧ (clearExceptCurrent pl).1.entries.length ≤ 1 := by
  constructor
  · exact removeBackwards_wf true pl _ hwf
  · have := congrArg List.length (clearExceptCurrent_ids pl hwf).2
    rw [List.length_map] at this
    rw [this]
    exact filter_eq_length_le_one _ _ hid

/-- Without a playlist directory `playlist_set_current` does nothing. -/
theorem setCurrent_noDir (pl : Playlist) (h : pl.playlistDir = none) : setCurrent pl = pl := by
  simp [setCurrent, h]

/-- With no entry whose filename is its playlist path, nothing changes. -/
theorem setCurrent_noMatch (pl : Playlist)
    (h : ∀ e ∈ pl.entries, e.playlistPath ≠ some e.filename) : setCurrent pl = pl := by
  have : pl.entries.find? (fun e => decide (e.playlistPath = some e.filename)) = none := by
    simpa using h
  simp only [setCurrent, this]
  split <;> rfl

/-- Otherwise the first entry whose filename is its playlist path becomes `current`. -/
theorem setCurrent_match (pl : Playlist) (hd : pl.playlistDir ≠ none) (i : Nat)
    (hi : i < pl.entries.length)
    (hm : pl.entries[i].playlistPath = some pl.entries[i].filename)
    (hfirst : ∀ (j : Nat) (hj : j < i),
      pl.entries[j].playlistPath ≠ some pl.entries[j].filename) :
    setCurrent pl = { pl with current := some pl.entries[i].id } := by
  have : pl.entries.find? (fun e => decide (e.playlistPath = some e.filename)) =
      some pl.entries[i] := by
    rw [List.find?_eq_some_iff_getElem]
    exact ⟨by simpa using hm, i, hi, rfl, fun j hj => by simpa using hfirst j hj⟩
  cases hdir : pl.playlistDir with
  | none => exact absurd hdir hd
  | some d => simp [setCurrent, hdir, this]

/-- Every field of an entry except its position. -/
def Entry.forgetIndex (e : Entry) : Entry := { e with plIndex := 0 }

theorem perm_eraseIdx {α : Type} (l : List α) (i : Nat) (h : i < l.length) :
    l.Perm (l[i] :: l.eraseIdx i) := by
  induction l generalizing i with
  | nil => simp at h
  | cons a l ih =>
    cases i with
    | zero => simp
    | succ i =>
      simp only [List.getElem_cons_succ, List.eraseIdx_cons_succ]
      exact ((ih i (by simpa using h)).cons a).trans (List.Perm.swap _ _ _)

/-- The array work of `playlist_move`: insert `entry` at `i`, then remove its
old slot `o`, which the insertion shifted to `o'` when `o ≥ i`. -/
theorem move_shape (l : List Entry) (entry : Entry) (i o o' : Nat)
    (hi : i ≤ l.length) (ho : o < l.length) (he : l[o] = entry)
    (ho' : o' = if o ≥ i then o + 1 else o) :
    ((l.insertIdx i entry).eraseIdx o').Perm l ∧
    ((l.insertIdx i entry).eraseIdx o').length = l.length ∧
    ∀ (k : Nat) (hk : k < ((l.insertIdx i entry).eraseIdx o').length),
      (∃ (j : Nat) (hj : j < l.length), ((l.insertIdx i entry).eraseIdx o')[k] = l[j]) ∧
      (k + 1 < min i o' ∨ max i o' < k →
        ∃ hk' : k < l.length, ((l.insertIdx i entry).eraseIdx o')[k] = l[k]) := by
  have hlt : o' < (l.insertIdx i entry).length := by grind
  have hget : (l.insertIdx i entry)[o'] = entry := by grind
  refine ⟨?_, by grind, ?_⟩
  · have h1 := perm_eraseIdx (l.insertIdx i entry) o' hlt
    rw [hget] at h1
    exact ((List.perm_insertIdx entry l hi).symm.trans h1).cons_inv.symm
  · intro k hk
    exact ⟨by grind, fun hrange => ⟨by grind, by grind⟩⟩

/-- `playlist_move` on live pointers, with the indexes as naturals: `i` is the
target slot and `o'` the entry's old slot after the insertion. -/
theorem move_eq (pl : Playlist) (entry : Entry) (at_ : Option Entry)
    (he : Live pl entry) (ha : ∀ a, at_ = some a → Live pl a) (hne : some entry ≠ at_) :
    ∃ i o' : Nat, i ≤ pl.entries.length ∧
      o' = (if entry.plIndex.toNat ≥ i then entry.plIndex.toNat + 1 else entry.plIndex.toNat) ∧
      move pl entry at_ =
        updateIndexes { pl with entries := (pl.entries.insertIdx i entry).eraseIdx o' }
          (min ((i : Int) - 1) ((o' : Int) - 1)) (max ((i : Int) + 1) ((o' : Int) + 1)) := by
  have h0 := he.1
  have key : ∀ index : Int, 0 ≤ index → index.toNat ≤ pl.entries.length →
      ∃ i o' : Nat, i ≤ pl.entries.length ∧
        o' = (if entry.plIndex.toNat ≥ i then entry.plIndex.toNat + 1
          else entry.plIndex.toNat) ∧
        index = (i : Int) ∧
        (if entry.plIndex ≥ index then entry.plIndex + 1 else entry.plIndex) = (o' : Int) := by
    intro index h0i hle
    refine ⟨index.toNat, _, hle, rfl, by omega, ?_⟩
    split <;> split <;> omega
  have finish : ∀ index : Int, 0 ≤ index → index.toNat ≤ pl.entries.length →
      ∃ i o' : Nat, i ≤ pl.entries.length ∧
        o' = (if entry.plIndex.toNat ≥ i then entry.plIndex.toNat + 1
          else entry.plIndex.toNat) ∧
        updateIndexes
            { pl with
              entries := (pl.entries.insertIdx index.toNat entry).eraseIdx
                (if entry.plIndex ≥ index then entry.plIndex + 1 else entry.plIndex).toNat }
            (min (index - 1)
              ((if entry.plIndex ≥ index then entry.plIndex + 1 else entry.plIndex) - 1))
            (max (index + 1)
              ((if entry.plIndex ≥ index then entry.plIndex + 1 else entry.plIndex) + 1)) =
          updateIndexes { pl with entries := (pl.entries.insertIdx i entry).eraseIdx o' }
            (min ((i : Int) - 1) ((o' : Int) - 1)) (max ((i : Int) + 1) ((o' : Int) + 1)) := by
    intro index h0i hle
    obtain ⟨i, o', hi, ho', e1, e2⟩ := key index h0i hle
    refine ⟨i, o', hi, ho', ?_⟩
    rw [e2, e1, Int.toNat_natCast, Int.toNat_natCast]
  cases at_ with
  | none =>
    simp only [move, reduceCtorEq, ↓reduceIte]
    exact finish _ (by omega) (by simp)
  | some a =>
    simp only [move, hne, ↓reduceIte]
    exact finish _ (ha a rfl).1 (by have := live_lt (ha a rfl); omega)

/-- `playlist_move` keeps every `pl_index` equal to its position. -/
theorem move_wf (pl : Playlist) (entry : Entry) (at_ : Option Entry) (hwf : WF pl)
    (he : Live pl entry) (ha : ∀ a, at_ = some a → Live pl a) : WF (move pl entry at_) := by
  by_cases hne : some entry = at_
  · simpa [move, hne] using hwf
  obtain ⟨i, o', hi, ho', heq⟩ := move_eq pl entry at_ he ha hne
  have hget : pl.entries[entry.plIndex.toNat]'(live_lt he) = entry :=
    (List.getElem?_eq_some_iff.mp he.2).2
  obtain ⟨-, hlen, hall⟩ :=
    move_shape pl.entries entry i entry.plIndex.toNat o' hi (live_lt he) hget ho'
  rw [heq]
  apply updateIndexes_wf_of
  intro k hk
  have hk' : k < pl.entries.length := by simpa [hlen] using hk
  obtain ⟨⟨j, hj, hrj⟩, hout⟩ := hall k hk
  refine ⟨by rw [hrj]; exact (hwf j hj).2, ?_⟩
  by_cases hrange : k + 1 < min i o' ∨ max i o' < k
  · obtain ⟨_, hsame⟩ := hout hrange
    exact Or.inl (by rw [hsame]; exact (hwf k hk').1)
  · have hstop : ¬ max ((i : Int) + 1) ((o' : Int) + 1) < 0 := by omega
    rw [ite_eq_right_of_eq_false _ _ (eq_false hstop)]
    dsimp only
    rw [hlen]
    exact Or.inr (by omega)

/-- `playlist_move` keeps the same entries. Only their `pl_index` may change. -/
theorem move_perm (pl : Playlist) (entry : Entry) (at_ : Option Entry)
    (he : Live pl entry) (ha : ∀ a, at_ = some a → Live pl a) :
    ((move pl entry at_).entries.map Entry.forgetIndex).Perm
      (pl.entries.map Entry.forgetIndex) := by
  by_cases hne : some entry = at_
  · simp [move, hne]
  obtain ⟨i, o', hi, ho', heq⟩ := move_eq pl entry at_ he ha hne
  have hget : pl.entries[entry.plIndex.toNat]'(live_lt he) = entry :=
    (List.getElem?_eq_some_iff.mp he.2).2
  rw [heq, updateIndexes_map _ _ _ _ (fun _ _ => rfl)]
  exact (move_shape pl.entries entry i entry.plIndex.toNat o' hi (live_lt he) hget ho').1.map _

/-- Distinct C pointers are distinct `id`s, none newer than `id_alloc`. -/
def FreshIds (pl : Playlist) : Prop :=
  (pl.entries.map (·.id)).Nodup ∧ ∀ id ∈ pl.entries.map (·.id), id ≤ pl.idAlloc

/-- `playlist_insert_at` gives the new entry an `id` no other entry has. -/
theorem insertAt_freshIds (pl : Playlist) (add : Entry) (at_ : Option Entry)
    (h : FreshIds pl) (ha : ∀ a, at_ = some a → Live pl a) : FreshIds (insertAt pl add at_) := by
  have key : ∀ index : Nat, index ≤ pl.entries.length → ∀ (x : Entry) (s e : Int),
      x.id = pl.idAlloc + 1 →
      FreshIds (updateIndexes
        { pl with entries := pl.entries.insertIdx index x, idAlloc := pl.idAlloc + 1 } s e) := by
    intro index hle x s e hx
    have hperm : ((pl.entries.insertIdx index x).map (·.id)).Perm
        ((pl.idAlloc + 1) :: pl.entries.map (·.id)) := by
      rw [← hx]
      exact (List.perm_insertIdx x pl.entries hle).map _
    unfold FreshIds
    rw [updateIndexes_map _ _ _ _ (fun _ _ => rfl)]
    constructor
    · rw [hperm.nodup_iff, List.nodup_cons]
      exact ⟨fun hmem => by have := h.2 _ hmem; omega, h.1⟩
    · intro id hid
      rcases List.mem_cons.mp (hperm.mem_iff.mp hid) with rfl | hmem
      · exact Nat.le_refl _
      · exact Nat.le_succ_of_le (h.2 _ hmem)
  cases at_ with
  | none => exact key _ (Nat.le_refl _) _ _ _ rfl
  | some a => exact key a.plIndex.toNat (by have := live_lt (ha a rfl); omega) _ _ _ rfl

end Mpv.Common.Playlist
