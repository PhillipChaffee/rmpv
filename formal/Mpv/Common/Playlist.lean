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
  deriving Repr, DecidableEq

structure Playlist where
  entries : List Entry
  /-- `id` of the C `current` entry. -/
  current : Option Nat
  currentWasReplaced : Bool
  playlistCompleted : Bool
  playlistStarted : Bool
  idAlloc : Nat
  deriving Repr

/-- Every entry is attached and knows its own position. -/
def WF (pl : Playlist) : Prop :=
  ∀ (i : Nat) (h : i < pl.entries.length),
    pl.entries[i].plIndex = i ∧ pl.entries[i].attached = true

/-- `e` is a live pointer into `pl`. -/
def Live (pl : Playlist) (e : Entry) : Prop :=
  0 ≤ e.plIndex ∧ pl.entries[e.plIndex.toNat]? = some e

-- mpv-cover: common/playlist.c playlist_entry_count paths=all hash=110c2efbe62f
def entryCount (pl : Playlist) : Int := pl.entries.length

-- mpv-cover: common/playlist.c playlist_entry_from_index paths=all hash=54c08dd62920
def entryFromIndex (pl : Playlist) (index : Int) : Option Entry :=
  if 0 ≤ index ∧ index < pl.entries.length then pl.entries[index.toNat]? else none

-- mpv-cover: common/playlist.c playlist_entry_to_index paths=all hash=0aa9a76a75f9
def entryToIndex (e : Option Entry) : Int :=
  match e with
  | none => -1
  | some e => if !e.attached then -1 else e.plIndex

-- mpv-cover: common/playlist.c playlist_update_indexes paths=all hash=ca8fa75c6842
def updateIndexes (pl : Playlist) (start stop : Int) : Playlist :=
  let s := max start 0
  let e := if stop < 0 then (pl.entries.length : Int) else min stop pl.entries.length
  { pl with entries := pl.entries.mapIdx fun n x =>
      if s ≤ (n : Int) ∧ (n : Int) < e then { x with plIndex := n } else x }

-- Paths 2, 5 and 9 are `mp_assert` failures, which abort; the model takes the
-- asserted conditions as hypotheses instead. Path 10 only grows the array.
-- mpv-cover: common/playlist.c playlist_insert_at paths=0,1,3,4,6,7,8 unobservable=10 hash=5b59e2deca02
def insertAt (pl : Playlist) (add : Entry) (at_ : Option Entry) : Playlist :=
  let index : Int := match at_ with
    | some a => a.plIndex
    | none => pl.entries.length
  let add' := { add with attached := true, plIndex := index, id := pl.idAlloc + 1 }
  let pl' := { pl with entries := pl.entries.insertIdx index.toNat add', idAlloc := pl.idAlloc + 1 }
  updateIndexes pl' index pl'.entries.length

-- Path 3 is an `mp_assert` failure, which aborts; the model takes the
-- asserted conditions as hypotheses instead.
-- mpv-cover: common/playlist.c playlist_entry_get_rel paths=0,1,2,4 hash=5fff8931ad8f
def entryGetRel (pl : Playlist) (e : Entry) (direction : Int) : Option Entry :=
  if !e.attached then none else entryFromIndex pl (e.plIndex + direction)

-- Paths 3 and 6 are `mp_assert` failures, which abort; the model takes the
-- asserted conditions as hypotheses instead.
-- mpv-cover: common/playlist.c playlist_remove paths=0,1,2,4,5 hash=ba66035fec9a
def remove (pl : Playlist) (entry : Entry) : Playlist × Entry :=
  let pl1 := if pl.current = some entry.id then
      { pl with current := (entryGetRel pl entry 1).map (·.id), currentWasReplaced := true }
    else pl
  let pl2 := { pl1 with entries := pl1.entries.eraseIdx entry.plIndex.toNat }
  (updateIndexes pl2 entry.plIndex (-1),
   { entry with attached := false, plIndex := -1, removed := true })

-- mpv-cover: common/playlist.c playlist_get_first paths=all hash=dbb2f8fcedea
def getFirst (pl : Playlist) : Option Entry :=
  if pl.entries.length ≠ 0 then pl.entries[0]? else none

-- mpv-cover: common/playlist.c playlist_get_last paths=all hash=f9e91f29ed9b
def getLast (pl : Playlist) : Option Entry :=
  if pl.entries.length ≠ 0 then pl.entries[pl.entries.length - 1]? else none

-- Paths 3 and 12 are `mp_assert` failures, which abort; the model takes the
-- asserted conditions as hypotheses instead.
-- mpv-cover: common/playlist.c playlist_get_next paths=0,1,2,4,5,6,7,8,9,10,11,13,14,15 hash=e80538901518
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

end Mpv.Common.Playlist
