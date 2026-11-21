import Mpv.Player.Playloop

/-!
Theorems about the playback time and position queries.

Lean gives `Float` no arithmetic laws: its operations are opaque to the proof
checker, so nothing here says what a sum or a comparison of two times is. These
theorems state the case structure instead: which branch a function takes, which
of its inputs the result is, and the integer bounds on chapter indexes.

A theorem that gives a result of `0` is about the model, whose zero is `+0.0`.
Of mpv it claims a zero of either sign; see `Playloop.lean`.
-/

namespace Mpv.Player.Playloop

/-! ## `get_time_length` -/

theorem getTimeLength_no_demuxer (c : Ctx) (h : c.demuxer = none) :
    getTimeLength c = nopts := by
  simp [getTimeLength, h]

theorem getTimeLength_known (c : Ctx) (d : Demuxer) (h : c.demuxer = some d)
    (known : d.duration ≥ 0) : getTimeLength c = d.duration := by
  simp [getTimeLength, h, known]

/-- The length is unknown, or it is the demuxer's duration and that passed the
`>= 0` test. -/
theorem getTimeLength_cases (c : Ctx) :
    getTimeLength c = nopts ∨
      ∃ d, c.demuxer = some d ∧ d.duration ≥ 0 ∧ getTimeLength c = d.duration := by
  unfold getTimeLength
  split
  · rename_i d h
    split
    · exact .inr ⟨d, h, by assumption, rfl⟩
    · exact .inl rfl
  · exact .inl rfl

/-! ## `get_start_time` -/

theorem getStartTime_no_demuxer (c : Ctx) (dir : Int) (h : c.demuxer = none) :
    getStartTime c dir = 0 := by
  simp [getStartTime, h]

/-- Playing forward with timestamps rebased, the start is 0 whatever the
demuxer reports. -/
theorem getStartTime_rebased_forward (c : Ctx) (dir : Int) (rebase : c.rebaseStartTime = true)
    (forward : 0 ≤ dir) : getStartTime c dir = 0 := by
  have : ¬ dir < 0 := by omega
  unfold getStartTime
  split <;> simp [rebase, this]

/-- Only the sign of the direction matters. -/
theorem getStartTime_forward (c : Ctx) (dir : Int) (forward : 0 ≤ dir) :
    getStartTime c dir = getStartTime c 0 := by
  have : ¬ dir < 0 := by omega
  simp [getStartTime, this]

theorem getStartTime_backward (c : Ctx) (dir : Int) (backward : dir < 0) :
    getStartTime c dir = getStartTime c (-1) := by
  simp [getStartTime, backward]

/-! ## `get_current_time` -/

theorem getCurrentTime_no_demuxer (c : Ctx) (h : c.demuxer = none) :
    getCurrentTime c = nopts := by
  simp [getCurrentTime, h]

/-- With no frame shown since the last seek, the time is the seek target. -/
theorem getCurrentTime_seeking (c : Ctx) (d : Demuxer) (h : c.demuxer = some d)
    (seeking : (c.playbackPts != nopts) = false) : getCurrentTime c = c.lastSeekPts := by
  simp [getCurrentTime, h, seeking]

theorem getCurrentTime_playing (c : Ctx) (d : Demuxer) (h : c.demuxer = some d)
    (playing : (c.playbackPts != nopts) = true) :
    getCurrentTime c = c.playbackPts * Float.ofInt c.playDir := by
  simp [getCurrentTime, h, playing]

/-! ## `get_playback_time` -/

/-- The result is 0, the current time unchanged, or the length. -/
theorem getPlaybackTime_cases (c : Ctx) :
    getPlaybackTime c = 0 ∨ getPlaybackTime c = getCurrentTime c ∨
      getPlaybackTime c = getTimeLength c := by
  simp only [getPlaybackTime, fclamp]
  repeat' split
  all_goals simp

/-- While a frame is showing, the time is never clamped to the length. -/
theorem getPlaybackTime_playing (c : Ctx) (playing : (c.playbackPts == nopts) = false) :
    getPlaybackTime c = 0 ∨ getPlaybackTime c = getCurrentTime c := by
  simp only [getPlaybackTime, playing, Bool.and_false, Bool.false_eq_true, ↓reduceIte]
  split
  · exact .inl rfl
  · exact .inr rfl

/-- The result is the model's 0 or fails the C test for a known negative time. -/
theorem getPlaybackTime_not_negative (c : Ctx) :
    getPlaybackTime c = 0 ∨
      ¬ ((getPlaybackTime c != nopts) = true ∧ getPlaybackTime c < 0) := by
  simp only [getPlaybackTime]
  generalize (if (getCurrentTime c != nopts && c.playbackPts == nopts) = true then _ else _) = cur
  split
  · exact .inl rfl
  · exact .inr (by assumption)

/-! ## `get_current_pos_ratio` -/

theorem getCurrentPosRatio_no_demuxer (playStart playEnd : Ctx → Float) (c : Ctx)
    (useRange : Bool) (h : c.demuxer = none) :
    getCurrentPosRatio playStart playEnd c useRange = -1 := by
  simp [getCurrentPosRatio, h]

/-- Without `use_range` the play range is never asked for. -/
theorem getCurrentPosRatio_ignores_range (playStart playEnd playStart' playEnd' : Ctx → Float)
    (c : Ctx) :
    getCurrentPosRatio playStart playEnd c false =
      getCurrentPosRatio playStart' playEnd' c false := by
  simp [getCurrentPosRatio]

/-! ## `get_current_chapter` -/

theorem chaptersStarted_le_size (c : Ctx) (time : Float) :
    chaptersStarted c time ≤ c.chapters.size :=
  Array.findIdx_le_size

/-- The loop passes over a chapter only when the time is not before its start. -/
theorem not_before_started_chapter (c : Ctx) (time : Float) (i : Nat)
    (h : i < chaptersStarted c time) :
    ¬ time < c.chapters[i]'(Nat.lt_of_lt_of_le h (chaptersStarted_le_size c time)) := by
  simpa using Array.not_of_lt_findIdx h

theorem getCurrentChapter_no_chapters (c : Ctx) (h : c.chapters.size = 0) :
    getCurrentChapter c = -2 := by
  simp [getCurrentChapter, h]

/-- A pending chapter seek is reported as it is, whatever the position. -/
theorem getCurrentChapter_pending_seek (c : Ctx) (some_chapters : c.chapters.size ≠ 0)
    (pending : c.lastChapterFlag = true) : getCurrentChapter c = c.lastChapterSeek := by
  simp [getCurrentChapter, some_chapters, pending]

/-- With chapters and no pending seek, the result is a chapter or -1, and never
less than the last chapter sought. -/
theorem getCurrentChapter_from_position (c : Ctx) (some_chapters : c.chapters.size ≠ 0)
    (not_pending : c.lastChapterFlag = false) :
    -1 ≤ getCurrentChapter c ∧ c.lastChapterSeek ≤ getCurrentChapter c ∧
      (c.lastChapterSeek < c.chapters.size → getCurrentChapter c < c.chapters.size) := by
  have := chaptersStarted_le_size c (getCurrentTime c)
  have result : getCurrentChapter c =
      max c.lastChapterSeek ((chaptersStarted c (getCurrentTime c) : Int) - 1) := by
    simp [getCurrentChapter, some_chapters, not_pending]
  rw [result]
  omega

/-- With chapters, the result is below the chapter count exactly when
`last_chapter_seek` is. This needs no assumption about that field. -/
theorem getCurrentChapter_lt_count_iff (c : Ctx) (some_chapters : c.chapters.size ≠ 0) :
    getCurrentChapter c < getChapterCount c ↔ c.lastChapterSeek < c.chapters.size := by
  have := chaptersStarted_le_size c (getCurrentTime c)
  simp only [getCurrentChapter, getChapterCount, some_chapters, ↓reduceIte]
  split <;> omega

/-- The result is at least -2 and less than the chapter count, provided
`last_chapter_seek` is. `low` is an invariant of mpv: the field is only ever
-2 or a chapter `chapter_start_time` accepted, which is at least -1. `high` is
not. It holds from the start of a file (the field is -2) and still holds right
after a chapter seek (command.c stores only an accepted chapter). It can fail
later: the seek clears `last_chapter_flag` but leaves
`last_chapter_seek`, and both a runtime `chapter-list` change and a demuxer
chapter update replace the chapter list without resetting the field. After a
seek to chapter 4, a new list of 2 chapters makes `get_current_chapter` return
4, at or above the count, until a seek of another kind or a new file resets the
field, or a new chapter seek overwrites it with a valid chapter.
`getCurrentChapter_lt_count_iff` is the fact that holds without the
hypotheses, and `getCurrentChapter_from_position` gives the lower bound -1
when no seek is pending. -/
theorem getCurrentChapter_bounds (c : Ctx) (low : -2 ≤ c.lastChapterSeek)
    (high : c.lastChapterSeek < c.chapters.size) :
    -2 ≤ getCurrentChapter c ∧ getCurrentChapter c < getChapterCount c := by
  have := chaptersStarted_le_size c (getCurrentTime c)
  simp only [getCurrentChapter, getChapterCount]
  repeat' split
  all_goals omega

/-! ## `chapter_start_time` -/

theorem chapterStartTime_before_first (c : Ctx) : chapterStartTime c (-1) = 0 := by
  simp [chapterStartTime]

theorem chapterStartTime_in_range (c : Ctx) (chapter : Int)
    (h : 0 ≤ chapter ∧ chapter < c.chapters.size) :
    chapterStartTime c chapter = c.chapters[chapter.toNat]'(by omega) := by
  have : chapter ≠ -1 := by omega
  simp [chapterStartTime, this, h]

theorem chapterStartTime_out_of_range (c : Ctx) (chapter : Int)
    (h : chapter < -1 ∨ (c.chapters.size : Int) ≤ chapter) :
    chapterStartTime c chapter = nopts := by
  have : chapter ≠ -1 := by omega
  have : ¬ (0 ≤ chapter ∧ chapter < c.chapters.size) := by omega
  simp [chapterStartTime, *]

/-! ## `get_chapter_count` -/

theorem getChapterCount_nonneg (c : Ctx) : 0 ≤ getChapterCount c := by
  simp [getChapterCount]

/-- `chapter_start_time` knows exactly the chapters from -1 up to the count. -/
theorem chapterStartTime_out_of_count (c : Ctx) (chapter : Int)
    (h : chapter < -1 ∨ getChapterCount c ≤ chapter) : chapterStartTime c chapter = nopts :=
  chapterStartTime_out_of_range c chapter h

/-! ## `get_cache_buffering_percentage` -/

theorem getCacheBufferingPercentage_no_demuxer (c : Ctx) (h : c.demuxer = none) :
    getCacheBufferingPercentage c = -1 := by
  simp [getCacheBufferingPercentage, h]

theorem getCacheBufferingPercentage_demuxer (c : Ctx) (d : Demuxer) (h : c.demuxer = some d) :
    getCacheBufferingPercentage c = c.cacheBuffer := by
  simp [getCacheBufferingPercentage, h]

end Mpv.Player.Playloop
