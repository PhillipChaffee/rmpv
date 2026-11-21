import Mpv.Common.Playlist

/-!
Theorems about shuffling, unshuffling, the playlist-path walks, the transfer
functions and `playlist_parse_file` in `common/playlist.c`.
-/

namespace Mpv.Common.Playlist

/-! ## Reindexing the whole array -/

/-- `playlist_update_indexes(pl, 0, -1)` rewrites every `pl_index`. -/
theorem updateIndexes_all (pl : Playlist) :
    (updateIndexes pl 0 (-1)).entries = pl.entries.mapIdx fun n e => { e with plIndex := n } := by
  apply List.ext_getElem
  · simp [updateIndexes]
  · intro i h1 h2
    have hi : i < pl.entries.length := by simpa [updateIndexes] using h1
    rw [updateIndexes_getElem, List.getElem_mapIdx]
    simp [hi]

theorem updateIndexes_all_wf (pl : Playlist) (h : ∀ e ∈ pl.entries, e.attached = true) :
    WF (updateIndexes pl 0 (-1)) := by
  apply updateIndexes_wf_of
  intro k hk
  exact ⟨h _ (List.getElem_mem hk), Or.inr ⟨by omega, by simp; omega⟩⟩

/-- Reindexing gives the same array for two arrays that differ only in `pl_index`. -/
theorem mapIdx_plIndex_congr {l₁ l₂ : List Entry}
    (h : l₁.map Entry.forgetIndex = l₂.map Entry.forgetIndex) :
    (l₁.mapIdx fun n e => { e with plIndex := n }) =
      l₂.mapIdx fun n e => { e with plIndex := n } := by
  have hlen : l₁.length = l₂.length := by simpa using congrArg List.length h
  apply List.ext_getElem
  · simpa using hlen
  · intro i h1 h2
    have hi : i < l₁.length := by simpa using h1
    have hget : l₁[i].forgetIndex = (l₂[i]'(hlen ▸ hi)).forgetIndex := by
      have := List.getElem_of_eq h (i := i) (by simpa using hi)
      simpa using this
    simp only [List.getElem_mapIdx]
    have key : ∀ a b : Entry, a.forgetIndex = b.forgetIndex →
        ({ a with plIndex := (i : Int) } : Entry) = { b with plIndex := (i : Int) } := by
      intro a b hab
      cases a; cases b
      simp only [Entry.forgetIndex, Entry.mk.injEq] at hab ⊢
      simp_all
    exact key _ _ hget

/-! ## `playlist_shuffle` -/

theorem swap_perm (l : List Entry) (i j : Nat) : (swap l i j).Perm l := by
  unfold swap
  split
  · rename_i a b ha hb
    obtain ⟨hi, rfl⟩ := List.getElem?_eq_some_iff.mp ha
    obtain ⟨hj, rfl⟩ := List.getElem?_eq_some_iff.mp hb
    exact List.set_set_perm hi hj
  · exact .refl _

/-- The swap loop of `playlist_shuffle` rearranges the array, whatever numbers
`mp_rand_in_range32` returns. -/
theorem swaps_perm (rand : Nat → Nat → Nat → Nat) (len : Nat) (ns : List Nat) (l : List Entry) :
    (ns.foldl (fun l n => swap l n (rand n n len)) l).Perm l := by
  induction ns generalizing l with
  | nil => exact .refl _
  | cons n ns ih => exact (ih _).trans (swap_perm _ _ _)

/-- The array after the first loop of `playlist_shuffle`. -/
def markOriginal (entries : List Entry) : List Entry :=
  entries.mapIdx fun n e => { e with originalIndex := n }

/-- `playlist_shuffle` as a rearrangement of the marked array, reindexed. -/
theorem shuffle_entries (rand : Nat → Nat → Nat → Nat) (pl : Playlist) :
    ∃ swapped : List Entry, swapped.Perm (markOriginal pl.entries) ∧
      (shuffle rand pl).entries = swapped.mapIdx fun n e => { e with plIndex := n } := by
  refine ⟨_, swaps_perm rand (markOriginal pl.entries).length
    (List.range ((markOriginal pl.entries).length - 1)) (markOriginal pl.entries), ?_⟩
  simp only [shuffle, updateIndexes_all, markOriginal]

/-- Every field of an entry except the two positions `playlist_shuffle` rewrites. -/
def Entry.forgetOrder (e : Entry) : Entry := { e with plIndex := 0, originalIndex := 0 }

/-- `playlist_shuffle` keeps the same entries. Only their `pl_index` and
`original_index` change. -/
theorem shuffle_perm (rand : Nat → Nat → Nat → Nat) (pl : Playlist) :
    ((shuffle rand pl).entries.map Entry.forgetOrder).Perm
      (pl.entries.map Entry.forgetOrder) := by
  obtain ⟨swapped, hperm, heq⟩ := shuffle_entries rand pl
  have h1 : (swapped.mapIdx fun n e => { e with plIndex := n }).map Entry.forgetOrder =
      swapped.map Entry.forgetOrder := by
    apply List.ext_getElem <;> simp [Entry.forgetOrder]
  have h2 : (markOriginal pl.entries).map Entry.forgetOrder =
      pl.entries.map Entry.forgetOrder := by
    apply List.ext_getElem <;> simp [Entry.forgetOrder, markOriginal]
  rw [heq, h1, ← h2]
  exact hperm.map _

/-- `playlist_shuffle` keeps every `pl_index` equal to its position. -/
theorem shuffle_wf (rand : Nat → Nat → Nat → Nat) (pl : Playlist) (hwf : WF pl) :
    WF (shuffle rand pl) := by
  unfold shuffle
  apply updateIndexes_all_wf
  intro e he
  have hmem := (swaps_perm rand _ _ _).mem_iff.mp he
  obtain ⟨i, hi, rfl⟩ := List.getElem_of_mem hmem
  have hi' : i < pl.entries.length := by simpa using hi
  simpa using (hwf i hi').2

/-! ## `playlist_unshuffle` -/

theorem unshuffle_entries (sort : (Entry → Entry → Int) → List Entry → List Entry)
    (pl : Playlist) :
    (unshuffle sort pl).entries =
      (if pl.entries.length ≠ 0 then sort cmpUnshuffle pl.entries else pl.entries).mapIdx
        fun n e => { e with plIndex := n } := by
  simp only [unshuffle, updateIndexes_all]

/-- `playlist_unshuffle` keeps the same entries when `qsort` rearranges the
array, as it does for any comparison. Only `pl_index` may change. -/
theorem unshuffle_perm (sort : (Entry → Entry → Int) → List Entry → List Entry) (pl : Playlist)
    (hsort : (sort cmpUnshuffle pl.entries).Perm pl.entries) :
    ((unshuffle sort pl).entries.map Entry.forgetIndex).Perm
      (pl.entries.map Entry.forgetIndex) := by
  have h1 : ∀ l : List Entry,
      (l.mapIdx fun n e => { e with plIndex := n }).map Entry.forgetIndex =
        l.map Entry.forgetIndex := by
    intro l
    apply List.ext_getElem <;> simp [Entry.forgetIndex]
  rw [unshuffle_entries, h1]
  split
  · exact hsort.map _
  · exact .refl _

/-- `playlist_unshuffle` keeps every `pl_index` equal to its position. -/
theorem unshuffle_wf (sort : (Entry → Entry → Int) → List Entry → List Entry) (pl : Playlist)
    (hwf : WF pl) (hsort : (sort cmpUnshuffle pl.entries).Perm pl.entries) :
    WF (unshuffle sort pl) := by
  unfold unshuffle
  apply updateIndexes_all_wf
  intro e he
  have hmem : e ∈ pl.entries := by
    dsimp only at he
    split at he
    · exact hsort.mem_iff.mp he
    · exact he
  obtain ⟨i, hi, rfl⟩ := List.getElem_of_mem hmem
  exact (hwf i hi).2

/-- With a known `original_index` on the left, `cmp_unshuffle` orders by it first. -/
theorem cmpUnshuffle_le (a b : Entry) (ha : 0 ≤ a.originalIndex) (h : cmpUnshuffle a b ≤ 0) :
    a.originalIndex ≤ b.originalIndex := by
  unfold cmpUnshuffle cmpInt at h
  split at h
  · split at h
    · omega
    · split at h <;> omega
  · omega

/-- Unshuffling a shuffled playlist restores the original order. `qsort` is
assumed to do its job on the shuffled array: return a rearrangement that is in
order by `cmp_unshuffle`. The comparison is consistent there, because every
entry has a known `original_index`. -/
theorem unshuffle_shuffle_entries (rand : Nat → Nat → Nat → Nat)
    (sort : (Entry → Entry → Int) → List Entry → List Entry) (pl : Playlist)
    (hperm : (sort cmpUnshuffle (shuffle rand pl).entries).Perm (shuffle rand pl).entries)
    (hsorted : (sort cmpUnshuffle (shuffle rand pl).entries).Pairwise
      fun a b => cmpUnshuffle a b ≤ 0) :
    (unshuffle sort (shuffle rand pl)).entries =
      pl.entries.mapIdx fun n e => { e with plIndex := n, originalIndex := n } := by
  obtain ⟨swapped, hswapped, heq⟩ := shuffle_entries rand pl
  have hforget : ∀ l : List Entry,
      (l.mapIdx fun n e => { e with plIndex := n }).map Entry.forgetIndex =
        l.map Entry.forgetIndex := by
    intro l
    apply List.ext_getElem <;> simp [Entry.forgetIndex]
  have htarget : (pl.entries.mapIdx fun n e => { e with plIndex := n, originalIndex := n }) =
      (markOriginal pl.entries).mapIdx fun n e => { e with plIndex := n } := by
    apply List.ext_getElem <;> simp [markOriginal]
  rw [unshuffle_entries, htarget]
  by_cases hlen : (shuffle rand pl).entries.length ≠ 0
  · rw [ite_eq_left_of_eq_true _ _ (eq_true hlen)]
    apply mapIdx_plIndex_congr
    -- Both sides are in order by `original_index`, and hold the same entries.
    have hp : ((sort cmpUnshuffle (shuffle rand pl).entries).map Entry.forgetIndex).Perm
        ((markOriginal pl.entries).map Entry.forgetIndex) := by
      refine (hperm.map _).trans ?_
      rw [heq, hforget]
      exact hswapped.map _
    have hindex : ∀ (i : Nat) (hi : i < ((markOriginal pl.entries).map Entry.forgetIndex).length),
        (((markOriginal pl.entries).map Entry.forgetIndex)[i]).originalIndex = i := by
      intro i hi
      simp [markOriginal, Entry.forgetIndex]
    have hknown : ∀ a ∈ sort cmpUnshuffle (shuffle rand pl).entries, 0 ≤ a.originalIndex := by
      intro a ha
      have hmem : a.forgetIndex ∈ (markOriginal pl.entries).map Entry.forgetIndex :=
        hp.mem_iff.mp (List.mem_map_of_mem ha)
      obtain ⟨i, hi, hget⟩ := List.getElem_of_mem hmem
      have := hindex i hi
      rw [hget] at this
      simp only [Entry.forgetIndex] at this
      omega
    apply List.Perm.eq_of_pairwise (le := fun a b => a.originalIndex ≤ b.originalIndex) _ _ _ hp
    · intro a b ha hb hab hba
      obtain ⟨i, hi, rfl⟩ := List.getElem_of_mem (hp.mem_iff.mp ha)
      obtain ⟨j, hj, rfl⟩ := List.getElem_of_mem hb
      rw [hindex i hi, hindex j hj] at hab hba
      have : i = j := by omega
      subst this
      rfl
    · rw [List.pairwise_map]
      refine hsorted.imp_of_mem ?_
      intro a b ha _ hab
      exact cmpUnshuffle_le a b (hknown a ha) hab
    · rw [List.pairwise_iff_getElem]
      intro i j hi hj hij
      rw [hindex i hi, hindex j hj]
      omega
  · have hnil : (shuffle rand pl).entries = [] :=
      List.eq_nil_of_length_eq_zero (by omega)
    have hempty : markOriginal pl.entries = [] := by
      rw [heq] at hnil
      have : swapped = [] := by simpa using hnil
      simpa [this] using hswapped.symm
    rw [ite_eq_right_of_eq_false _ _ (eq_false hlen), hnil, hempty]

/-- On a well-formed playlist, shuffling and then unshuffling changes one
thing: each entry now records its index as `original_index`. -/
theorem unshuffle_shuffle (rand : Nat → Nat → Nat → Nat)
    (sort : (Entry → Entry → Int) → List Entry → List Entry) (pl : Playlist) (hwf : WF pl)
    (hperm : (sort cmpUnshuffle (shuffle rand pl).entries).Perm (shuffle rand pl).entries)
    (hsorted : (sort cmpUnshuffle (shuffle rand pl).entries).Pairwise
      fun a b => cmpUnshuffle a b ≤ 0) :
    unshuffle sort (shuffle rand pl) = { pl with entries := markOriginal pl.entries } := by
  have hentries := unshuffle_shuffle_entries rand sort pl hperm hsorted
  have hsame : (pl.entries.mapIdx fun n e => { e with plIndex := n, originalIndex := n }) =
      markOriginal pl.entries := by
    apply List.ext_getElem
    · simp [markOriginal]
    · intro i h1 h2
      have hi : i < pl.entries.length := by simpa using h1
      have := (hwf i hi).1
      simp only [List.getElem_mapIdx, markOriginal]
      rw [← this]
  rw [hsame] at hentries
  have hrest : unshuffle sort (shuffle rand pl) =
      { pl with entries := (unshuffle sort (shuffle rand pl)).entries } := by
    simp [unshuffle, shuffle, updateIndexes]
  rw [hrest, hentries]

/-! ## The playlist-path walks -/

theorem firstWithPath_path (pl : Playlist) (path : String) (fuel : Nat) (e : Entry)
    (h : e.playlistPath = some path) : (firstWithPath pl path fuel e).playlistPath = some path := by
  induction fuel generalizing e with
  | zero => exact h
  | succ fuel ih =>
    unfold firstWithPath
    split
    · split
      · rename_i hprev
        exact ih _ hprev
      · exact h
    · exact h

/-- `playlist_get_first_in_same_playlist` returns an entry with the playlist
path of the entry it was given. -/
theorem getFirstInSamePlaylist_path (strip : String → String) (pl : Playlist) (e r : Entry)
    (current : Option String)
    (h : getFirstInSamePlaylist strip pl (some e) current = some r) :
    r.playlistPath = e.playlistPath := by
  unfold getFirstInSamePlaylist at h
  split at h
  · simp at h
  · rename_i e' he
    cases he
    split at h
    · simp_all
    · rename_i path hpath
      split at h
      · simp_all
      · have := firstWithPath_path pl path pl.entries.length e hpath
        simp_all

/-- With no next entry there is no next playlist. -/
theorem getFirstInNextPlaylist_none (strip : String → String) (pl : Playlist) (direction : Int)
    (h : getNext pl direction = none) : getFirstInNextPlaylist strip pl direction = none := by
  simp [getFirstInNextPlaylist, h]

/-! ## The transfer functions -/

/-- `playlist_transfer_entries_to` grows the destination by the source's count
and leaves the source empty. -/
theorem transferEntriesTo_count (pl : Playlist) (dstIndex : Int) (source : Playlist) :
    entryCount (transferEntriesTo pl dstIndex source).1 = entryCount pl + entryCount source ∧
    (transferEntriesTo pl dstIndex source).2.1.entries = [] := by
  simp only [transferEntriesTo, entryCount, updateIndexes_length, List.length_append,
    List.length_take, List.length_drop, List.length_mapIdx, and_true]
  omega

/-- `playlist_transfer_entries` grows the destination by the source's count and
leaves the source empty. -/
theorem transferEntries_count (pl source : Playlist) :
    entryCount (transferEntries pl source).1 = entryCount pl + entryCount source ∧
    (transferEntries pl source).2.1.entries = [] :=
  transferEntriesTo_count pl _ source

theorem appendEntries_count (pl source : Playlist) :
    entryCount (appendEntries pl source).1 = entryCount pl + entryCount source ∧
    (appendEntries pl source).2.1.entries = [] :=
  transferEntriesTo_count pl _ source

/-- `playlist_transfer_entries_to` keeps every `pl_index` equal to its position.
The bounds on `dst_index` are what the assertion in `MP_TARRAY_INSERT_N_AT`
checks. The source need not be well-formed. -/
theorem transferEntriesTo_wf (pl : Playlist) (dstIndex : Int) (source : Playlist) (hwf : WF pl)
    (h0 : 0 ≤ dstIndex) (hle : dstIndex ≤ pl.entries.length) :
    WF (transferEntriesTo pl dstIndex source).1 := by
  obtain ⟨d, rfl⟩ := Int.eq_ofNat_of_zero_le h0
  have hd : d ≤ pl.entries.length := by omega
  have hwf' : ∀ (i : Nat) (h : i < pl.entries.length),
      pl.entries[i].plIndex = i ∧ pl.entries[i].attached = true := hwf
  show WF (updateIndexes _ _ _)
  apply updateIndexes_wf_of
  intro k hk
  simp only [Int.toNat_natCast, List.length_append, List.length_take, List.length_drop,
    List.length_mapIdx] at hk ⊢
  by_cases h1 : k < d
  · have := hwf' k (by omega)
    refine ⟨?_, Or.inl ?_⟩ <;> grind
  · by_cases h2 : k < d + source.entries.length
    · refine ⟨?_, Or.inl ?_⟩ <;> grind
    · have := hwf' (k - source.entries.length) (by omega)
      refine ⟨by grind, Or.inr ⟨by omega, ?_⟩⟩
      simp only [Int.reduceNeg, Int.reduceLT, ↓reduceIte]
      omega

/-- `playlist_append_entries` keeps every `pl_index` equal to its position. -/
theorem appendEntries_wf (pl source : Playlist) (hwf : WF pl) :
    WF (appendEntries pl source).1 :=
  transferEntriesTo_wf pl _ source hwf (by omega) (by omega)

/-! ## `playlist_parse_file` -/

/-- A playlist `playlist_parse_file` returns is well-formed, and every entry
has the parsed file as its playlist path. -/
theorem parseFile_wf (getUserPath : String → String)
    (demuxOpenUrl : String → String → Int → Option Demuxer) (file : String) (pl : Playlist)
    (h : parseFile getUserPath demuxOpenUrl file = some pl) :
    WF pl ∧ ∀ e ∈ pl.entries, e.playlistPath = some file := by
  unfold parseFile at h
  split at h
  · simp at h
  · split at h
    · simp at h
    · rename_i playlist _
      cases h
      have hzero : ∀ source : Playlist,
          transferEntries Playlist.zeroed source = transferEntriesTo Playlist.zeroed 0 source := by
        intro source
        simp [transferEntries, Playlist.zeroed]
      rw [hzero]
      refine ⟨transferEntriesTo_wf _ 0 _ (fun i hi => by simp [Playlist.zeroed] at hi)
        (by omega) (by simp [Playlist.zeroed]), ?_⟩
      intro e he
      have hpaths : (transferEntriesTo Playlist.zeroed 0
          (populatePlaylistPath playlist file)).1.entries.map (·.playlistPath) =
          playlist.entries.map fun _ => some file := by
        show (updateIndexes _ _ _).entries.map _ = _
        rw [updateIndexes_map _ _ _ _ (fun _ _ => rfl)]
        apply List.ext_getElem <;> simp [Playlist.zeroed, populatePlaylistPath]
      have hmem : e.playlistPath ∈ playlist.entries.map fun _ => some file := by
        rw [← hpaths]
        exact List.mem_map_of_mem he
      simp only [List.mem_map] at hmem
      obtain ⟨_, _, heq⟩ := hmem
      exact heq.symm

end Mpv.Common.Playlist
