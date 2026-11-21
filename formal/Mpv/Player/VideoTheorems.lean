import Mpv.Player.Video

/-!
Theorems about the A/V sync arithmetic.

`Float` operations are opaque to the proof checker, so nothing here says what a
drift or a speed is as a number. These theorems state the case structure: which
branch runs, which fields a function writes, and the integer facts about the
frame history.

A theorem that gives a result or a field of `0` is about the model, whose zero
is `+0.0`. Of mpv it claims a zero of either sign; see `Playloop.lean`.
-/

namespace Mpv.Player.Video

open Mpv.Player.Playloop (nopts fclamp)
open Mpv.Player.Audio (writtenAudioPts playingAudioPts)

/-! ## `adjust_sync` -/

/-- While audio is not playing, nothing is corrected. -/
theorem adjustSync_not_playing (c : Ctx) (vPts frameTime : Float)
    (h : c.audioStatus ≠ statusPlaying) : adjustSync c vPts frameTime = c := by
  simp [adjustSync, h]

/-- Under display sync the running total of corrections is cleared. -/
theorem adjustSync_display_sync (c : Ctx) (vPts frameTime : Float)
    (playing : c.audioStatus = statusPlaying) (active : c.displaySyncActive = true) :
    (adjustSync c vPts frameTime).totalAvsyncChange = 0 := by
  simp [adjustSync, playing, active]

/-- Otherwise the delay and the running total move by the same step. -/
theorem adjustSync_same_step (c : Ctx) (vPts frameTime : Float)
    (playing : c.audioStatus = statusPlaying) (inactive : c.displaySyncActive = false) :
    ∃ change, (adjustSync c vPts frameTime).delay = c.delay + change ∧
      (adjustSync c vPts frameTime).totalAvsyncChange = c.totalAvsyncChange + change := by
  simp only [adjustSync, playing, inactive]
  exact ⟨_, rfl, rfl⟩

/-- Only `delay` and `total_avsync_change` are written. -/
theorem adjustSync_writes (c : Ctx) (vPts frameTime : Float) :
    adjustSync c vPts frameTime =
      { c with delay := (adjustSync c vPts frameTime).delay
               totalAvsyncChange := (adjustSync c vPts frameTime).totalAvsyncChange } := by
  unfold adjustSync
  split <;> rfl

/-! ## `update_av_diff` -/

/-- Unless audio and video are both playing, the difference is reset to 0. -/
theorem updateAvDiff_not_playing (aoDelay : Float) (c : Ctx) (offset : Float)
    (h : c.audioStatus ≠ statusPlaying ∨ c.videoStatus ≠ statusPlaying) :
    updateAvDiff aoDelay c offset = { c with lastAvDifference := 0 } := by
  simp [updateAvDiff, h]

/-- With sparse video, the difference is reset to 0. -/
theorem updateAvDiff_sparse (aoDelay : Float) (c : Ctx) (offset : Float) (vo : VoChain)
    (chain : c.voChain = some vo) (sparse : vo.isSparse = true) :
    (updateAvDiff aoDelay c offset).lastAvDifference = 0 := by
  unfold updateAvDiff
  split
  · rfl
  · simp [chain, sparse]

/-- The desync warning flag is never cleared. -/
theorem updateAvDiff_keeps_warning (aoDelay : Float) (c : Ctx) (offset : Float)
    (shown : c.dropMessageShown = true) :
    (updateAvDiff aoDelay c offset).dropMessageShown = true := by
  unfold updateAvDiff
  repeat' split
  all_goals simp [shown]

/-- The flag is set only by a difference that passes the C test `fabs > 0.5`. -/
theorem updateAvDiff_warning (aoDelay : Float) (c : Ctx) (offset : Float)
    (fresh : c.dropMessageShown = false)
    (shown : (updateAvDiff aoDelay c offset).dropMessageShown = true) :
    (updateAvDiff aoDelay c offset).lastAvDifference.abs > 0.5 := by
  unfold updateAvDiff at shown ⊢
  repeat' split at shown
  all_goals simp_all

/-- Only `last_av_difference` and `drop_message_shown` are written. -/
theorem updateAvDiff_writes (aoDelay : Float) (c : Ctx) (offset : Float) :
    updateAvDiff aoDelay c offset =
      { c with lastAvDifference := (updateAvDiff aoDelay c offset).lastAvDifference
               dropMessageShown := (updateAvDiff aoDelay c offset).dropMessageShown } := by
  unfold updateAvDiff
  repeat' split
  all_goals rfl

/-! ## `calc_average_frame_duration` -/

theorem positiveDurations_length_le (c : Ctx) :
    (positiveDurations c).length ≤ c.pastFrames.size := by
  simpa [positiveDurations] using
    List.length_filter_le (fun dur : Float => !decide (dur ≤ 0))
      (c.pastFrames.toList.map (·.approxDuration))

/-- With no frame of positive duration, and so with no history, the average
is 0. -/
theorem calcAverageFrameDuration_none (c : Ctx) (h : positiveDurations c = []) :
    calcAverageFrameDuration c = 0 := by
  simp [calcAverageFrameDuration, meanOr, h]

theorem calcAverageFrameDuration_empty (c : Ctx) (h : c.pastFrames = #[]) :
    calcAverageFrameDuration c = 0 :=
  calcAverageFrameDuration_none c (by simp [positiveDurations, h])

/-! ## `calc_best_speed` -/

/-- With no factor to try, there is no speed. -/
theorem calcBestSpeed_no_factor (vsync frame maxChange : Float) (maxFactor : Int)
    (h : maxFactor ≤ 0) : calcBestSpeed vsync frame maxChange maxFactor = -1 := by
  have : maxFactor.toNat = 0 := by omega
  simp [calcBestSpeed, this]

/-- The result is -1, or the scale of some factor from 1 to `maxFactor`, and
that scale passed the C test `fabs(scale - 1) <= max_change`. -/
theorem calcBestSpeed_cases (vsync frame maxChange : Float) (maxFactor : Int) :
    calcBestSpeed vsync frame maxChange maxFactor = -1 ∨
      ∃ factor : Nat, 1 ≤ factor ∧ (factor : Int) ≤ maxFactor ∧
        speedForFactor (frame / vsync) maxChange factor =
          some (calcBestSpeed vsync frame maxChange maxFactor) ∧
        (calcBestSpeed vsync frame maxChange maxFactor - 1).abs ≤ maxChange := by
  simp only [calcBestSpeed]
  cases found : (List.range maxFactor.toNat).findSome?
      fun n => speedForFactor (frame / vsync) maxChange (n + 1) with
  | none => exact .inl rfl
  | some scale =>
    obtain ⟨n, inRange, isScale⟩ := List.exists_of_findSome?_eq_some found
    have bound := List.mem_range.mp inRange
    refine .inr ⟨n + 1, by omega, by omega, isScale, ?_⟩
    simp only [speedForFactor] at isScale
    split at isScale
    · rename_i within
      cases isScale
      exact within
    · cases isScale

/-! ## `find_best_speed` -/

/-- With no frame of positive duration, playback stays at normal speed. -/
theorem findBestSpeed_none (c : Ctx) (vsync : Float) (h : positiveDurations c = []) :
    findBestSpeed c vsync = 1 := by
  simp [findBestSpeed, meanOr, h]

theorem findBestSpeed_empty (c : Ctx) (vsync : Float) (h : c.pastFrames = #[]) :
    findBestSpeed c vsync = 1 :=
  findBestSpeed_none c vsync (by simp [positiveDurations, h])

/-! ## `using_spdif_passthrough` -/

theorem usingSpdifPassthrough_no_chain (aoFormat : Int) (isPcm : Int → Bool) (c : Ctx)
    (h : c.audio.aoChain = none) : usingSpdifPassthrough aoFormat isPcm c = false := by
  simp [usingSpdifPassthrough, h]

/-- Without an audio output the format is never asked for. -/
theorem usingSpdifPassthrough_no_ao (aoFormat : Int) (isPcm : Int → Bool) (c : Ctx)
    (chain : Audio.AoChain) (h : c.audio.aoChain = some chain) (closed : chain.ao = false) :
    usingSpdifPassthrough aoFormat isPcm c = false := by
  simp [usingSpdifPassthrough, h, closed]

theorem usingSpdifPassthrough_ao (aoFormat : Int) (isPcm : Int → Bool) (c : Ctx)
    (chain : Audio.AoChain) (h : c.audio.aoChain = some chain) (opened : chain.ao = true) :
    usingSpdifPassthrough aoFormat isPcm c = !isPcm aoFormat := by
  simp [usingSpdifPassthrough, h, opened]

/-! ## `adjust_audio_drift_compensation` -/

/-- Outside the two resampling modes, or while audio is not playing, audio
follows video and the filter state is cleared. -/
theorem adjustAudioDriftCompensation_off (c : Ctx) (duration : Float)
    (h : (c.videoSync ≠ vsDispResample ∧ c.videoSync ≠ vsDispTempo) ∨
      c.audioStatus ≠ statusPlaying) :
    adjustAudioDriftCompensation c duration =
      { c with speedFactorA := c.speedFactorV, audioDriftCompensation := 0, avdFiltered := 0 } := by
  simp [adjustAudioDriftCompensation, h]

/-- While compensating, the stored compensation and the audio speed factor
come from one value: the first is it less 1, the second is it times the video
speed factor. -/
theorem adjustAudioDriftCompensation_on (c : Ctx) (duration : Float)
    (mode : c.videoSync = vsDispResample ∨ c.videoSync = vsDispTempo)
    (playing : c.audioStatus = statusPlaying) :
    ∃ comp, (adjustAudioDriftCompensation c duration).audioDriftCompensation = comp - 1.0 ∧
      (adjustAudioDriftCompensation c duration).speedFactorA = comp * c.speedFactorV := by
  have : ¬ ((c.videoSync ≠ vsDispResample ∧ c.videoSync ≠ vsDispTempo) ∨
      c.audioStatus ≠ statusPlaying) := by
    rcases mode with mode | mode <;> simp [mode, playing, vsDispResample, vsDispTempo]
  simp only [adjustAudioDriftCompensation, this, ↓reduceIte]
  exact ⟨_, rfl, rfl⟩

/-- The video speed factor is read, never written. -/
theorem adjustAudioDriftCompensation_keeps_video (c : Ctx) (duration : Float) :
    (adjustAudioDriftCompensation c duration).speedFactorV = c.speedFactorV := by
  unfold adjustAudioDriftCompensation
  split <;> rfl

/-! ## `calculate_frame_duration` -/

/-- With fewer than two frames queued, the duration is the container's. -/
theorem nextFrameDuration_single (c : Ctx) (demuxDuration : Float) (h : c.nextFrames.size < 2) :
    nextFrameDuration c demuxDuration = demuxDuration := by
  have : c.nextFrames[1]? = none := by simp; omega
  unfold nextFrameDuration
  split
  · simp_all
  · rfl

/-- The duration is the container's, or the gap between the next two frames,
whose timestamps passed the C test `pts1 >= pts0`. -/
theorem nextFrameDuration_cases (c : Ctx) (demuxDuration : Float) :
    nextFrameDuration c demuxDuration = demuxDuration ∨
      ∃ pts0 pts1, c.nextFrames[0]? = some pts0 ∧ c.nextFrames[1]? = some pts1 ∧ pts1 ≥ pts0 ∧
        nextFrameDuration c demuxDuration = pts1 - pts0 := by
  unfold nextFrameDuration
  split
  · rename_i pts0 pts1 h0 h1
    split
    · rename_i ordered
      simp only [Bool.and_eq_true, decide_eq_true_eq] at ordered
      exact .inr ⟨pts0, pts1, h0, h1, ordered.2, rfl⟩
    · exact .inl rfl
  · exact .inl rfl

/-- The newest frame is never one of the frames averaged. -/
theorem similarDurations_length_lt (c : Ctx) (duration : Float) (h : 0 < c.pastFrames.size) :
    (similarDurations c duration).length < c.pastFrames.size := by
  have := (List.takeWhile_sublist
    (fun dur : Float => !(decide (dur ≤ 0) || decide ((dur - duration).abs ≥ ptsRoundingTolerance)))
    (l := (c.pastFrames.toList.drop 1).map (·.duration))).length_le
  simp only [List.length_map, List.length_drop, Array.length_toList] at this
  simp only [similarDurations]
  omega

/-- The history keeps its length. -/
theorem calculateFrameDuration_size (c : Ctx) :
    (calculateFrameDuration c).pastFrames.size = c.pastFrames.size := by
  unfold calculateFrameDuration
  split <;> simp

/-- Only the newest frame is written. -/
theorem calculateFrameDuration_older (c : Ctx) (n : Nat) (older : 0 < n) :
    (calculateFrameDuration c).pastFrames[n]? = c.pastFrames[n]? := by
  have : ¬ 0 = n := by omega
  unfold calculateFrameDuration
  split
  · rfl
  · simp [Array.getElem?_modify, this]

/-- Nothing but the history is written. -/
theorem calculateFrameDuration_writes (c : Ctx) :
    calculateFrameDuration c = { c with pastFrames := (calculateFrameDuration c).pastFrames } := by
  unfold calculateFrameDuration
  split <;> rfl

/-- The newest frame's duration is the gap to the frame after next when that
is known, else the container's frame time, or -1 when the container has no
positive frame rate. -/
theorem calculateFrameDuration_duration (c : Ctx) (vo : VoChain) (frame : Frame)
    (chain : c.voChain = some vo) (newest : c.pastFrames[0]? = some frame) :
    ((calculateFrameDuration c).pastFrames[0]?.map (·.duration)) =
      some (nextFrameDuration c (if vo.containerFps > 0 then 1.0 / vo.containerFps else -1)) := by
  simp [calculateFrameDuration, chain, Array.getElem?_modify, newest]

end Mpv.Player.Video
