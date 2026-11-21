import Mpv.Player.Audio

/-!
Theorems about the gain and audio timestamp functions.

`Float` and `Float32` operations are opaque to the proof checker, so nothing
here says what a gain is as a number. These theorems state the case structure:
which inputs a result depends on and which branch produces it.
-/

namespace Mpv.Player.Audio

open Mpv.Player.Playloop (nopts)

/-! ## `db_gain` -/

/-- `db_gain` is one call of the C library's `pow`. That function is opaque
here, so this is all a theorem can say; the value rests on the fidelity
check. -/
theorem dbGain_pow (db : Float) : dbGain db = Float.pow 10.0 (db / 20.0) := rfl

/-! ## `compute_replaygain` -/

theorem activeReplaygain_off (g : Gain) (off : g.rgainMode = 0) : activeReplaygain g = none := by
  simp [activeReplaygain, off]

theorem activeReplaygain_no_track (g : Gain) (h : g.track = none) :
    activeReplaygain g = none := by
  simp [activeReplaygain, h]

theorem activeReplaygain_no_data (g : Gain) (h : g.track = some none) :
    activeReplaygain g = none := by
  simp [activeReplaygain, h]

/-- With no replaygain data in use and no fallback, the factor is 1. -/
theorem computeReplaygain_none (g : Gain) (inactive : activeReplaygain g = none)
    (unset : (g.rgainFallback != 0) = false) : computeReplaygain g = 1.0 := by
  simp [computeReplaygain, inactive, unset]

/-- With no replaygain data in use, a fallback gain applies as is. -/
theorem computeReplaygain_fallback (g : Gain) (inactive : activeReplaygain g = none)
    (set : (g.rgainFallback != 0) = true) :
    computeReplaygain g = (dbGain g.rgainFallback.toFloat).toFloat32 := by
  simp [computeReplaygain, inactive, set]

/-- With replaygain off, the track's tags are never read. -/
theorem computeReplaygain_off_ignores_track (g : Gain) (track : Option (Option Replaygain))
    (off : g.rgainMode = 0) :
    computeReplaygain { g with track } = computeReplaygain g := by
  simp [computeReplaygain, activeReplaygain, off]

/-- In track mode with clipping allowed, the factor is the track gain plus the
preamp; the peaks and the album gain are not read. -/
theorem computeReplaygain_track_clip (g : Gain) (rg : Replaygain) (track : g.track = some (some rg))
    (mode : g.rgainMode = 1) (clip : g.rgainClip = true) :
    computeReplaygain g = (dbGain (rg.trackGain + g.rgainPreamp).toFloat).toFloat32 := by
  simp [computeReplaygain, activeReplaygain, track, mode, clip]

/-- In any mode other than off and track, the album gain is used. -/
theorem computeReplaygain_album_clip (g : Gain) (rg : Replaygain) (track : g.track = some (some rg))
    (on : g.rgainMode ≠ 0) (album : g.rgainMode ≠ 1) (clip : g.rgainClip = true) :
    computeReplaygain g = (dbGain (rg.albumGain + g.rgainPreamp).toFloat).toFloat32 := by
  simp [computeReplaygain, activeReplaygain, track, on, album, clip]

/-! ## `audio_get_gain` -/

/-- Muted, the gain is 0 whatever the volume and replaygain are. -/
theorem audioGetGain_muted (g : Gain) (muted : g.softvolMute = true) : audioGetGain g = 0.0 := by
  simp [audioGetGain, muted]

/-- With replaygain off, the gain does not depend on the track. -/
theorem audioGetGain_off_ignores_track (g : Gain) (track : Option (Option Replaygain))
    (off : g.rgainMode = 0) : audioGetGain { g with track } = audioGetGain g := by
  simp [audioGetGain, computeReplaygain, activeReplaygain, off]

/-! ## `written_audio_pts` -/

theorem writtenAudioPts_no_chain (c : Ctx) (h : c.aoChain = none) : writtenAudioPts c = nopts := by
  simp [writtenAudioPts, h]

theorem writtenAudioPts_chain (c : Ctx) (chain : AoChain) (h : c.aoChain = some chain) :
    writtenAudioPts c = chain.lastOutPts := by
  simp [writtenAudioPts, h]

/-! ## `playing_audio_pts` -/

/-- Without an audio output, the playing position is the written one and the
output's delay is not used. -/
theorem playingAudioPts_no_ao (aoDelay : Float) (c : Ctx) (h : c.ao = false) :
    playingAudioPts aoDelay c = writtenAudioPts c := by
  simp [playingAudioPts, h]

/-- An unknown written position stays unknown: the delay is not subtracted
from the sentinel. -/
theorem playingAudioPts_unknown (aoDelay : Float) (c : Ctx)
    (unknown : (writtenAudioPts c == nopts) = true) :
    playingAudioPts aoDelay c = writtenAudioPts c := by
  simp [playingAudioPts, unknown]

/-- The playing position is the written one, or that less the delay scaled by
the audio speed. -/
theorem playingAudioPts_cases (aoDelay : Float) (c : Ctx) :
    playingAudioPts aoDelay c = writtenAudioPts c ∨
      (c.ao = true ∧
        playingAudioPts aoDelay c = writtenAudioPts c - c.audioSpeed * aoDelay) := by
  simp only [playingAudioPts]
  split
  · exact .inl rfl
  · rename_i h
    simp only [Bool.or_eq_true, Bool.not_eq_true', not_or, Bool.not_eq_false] at h
    exact .inr ⟨h.2, rfl⟩

end Mpv.Player.Audio
