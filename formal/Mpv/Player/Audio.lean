import Mpv.Player.Playloop

/-!
A model of the gain and audio timestamp arithmetic in mpv's `player/audio.c`.

A C `float` is a Lean `Float32` and a C `double` is a Lean `Float`. Each
definition converts between the two exactly where C does: the gain functions
keep their running value in a `float` but call `pow` and divide in `double`, so
every step here rounds to `Float32` at the same point.

These functions share no field with the playloop queries, so they have their
own structures instead of more fields on `Playloop.Ctx`.
-/

namespace Mpv.Player.Audio

open Mpv.Player.Playloop (nopts)

/-- The `struct ao_chain` fields the model reads. -/
structure AoChain where
  /-- `last_out_pts`. -/
  lastOutPts : Float
  /-- Whether `ao` is not `NULL`. Read by `using_spdif_passthrough` in
  `player/video.c`. -/
  ao : Bool

/-- The `struct MPContext` fields the audio timestamp functions read. -/
structure Ctx where
  /-- `ao_chain`; `none` is `NULL`. -/
  aoChain : Option AoChain
  /-- Whether `ao` is not `NULL`. A different field from `ao_chain->ao`. -/
  ao : Bool
  /-- `audio_speed`. -/
  audioSpeed : Float

/-- `struct replaygain_data`. -/
structure Replaygain where
  trackGain : Float32
  trackPeak : Float32
  albumGain : Float32
  albumPeak : Float32

/-- What the gain functions read through `struct MPContext`: options, and the
replaygain data of the selected audio track. -/
structure Gain where
  /-- `opts->softvol_volume`, in percent. -/
  softvolVolume : Float32
  /-- `opts->softvol_gain`, in dB. -/
  softvolGain : Float32
  /-- `opts->softvol_mute`. -/
  softvolMute : Bool
  /-- `opts->rgain_mode`: 0 is off, 1 is track gain, anything else album gain. -/
  rgainMode : Int
  /-- `opts->rgain_preamp`, in dB. -/
  rgainPreamp : Float32
  /-- `opts->rgain_clip`: when set, clipping is allowed. -/
  rgainClip : Bool
  /-- `opts->rgain_fallback`, in dB. -/
  rgainFallback : Float32
  /-- `current_track[0][STREAM_AUDIO]`, as the `replaygain_data` of its codec.
  The outer `none` is no track, the inner `none` a track without the data. -/
  track : Option (Option Replaygain)

/-- A gain in dB as a factor. The result is whatever the C library's `pow`
returns. -/
-- mpv-cover: player/audio.c db_gain paths=all hash=e4122fc2da3e
def dbGain (db : Float) : Float := Float.pow 10.0 (db / 20.0)

/-- The replaygain data that applies: the track's, unless replaygain is off. -/
def activeReplaygain (g : Gain) : Option Replaygain :=
  if g.rgainMode ≠ 0 then g.track.join else none

/-- The replaygain factor: the track's or album's gain plus the preamp, limited
so that the peak does not clip, or the fallback gain when no data applies. -/
-- mpv-cover: player/audio.c compute_replaygain paths=all hash=f6f50e24aae9
def computeReplaygain (g : Gain) : Float32 :=
  match activeReplaygain g with
  | some rg =>
    let (gain, peak) :=
      if g.rgainMode = 1 then (rg.trackGain, rg.trackPeak) else (rg.albumGain, rg.albumPeak)
    let gain := gain + g.rgainPreamp
    let rgain := (dbGain gain.toFloat).toFloat32
    if !g.rgainClip then
      -- `MPMIN(rgain, 1.0 / peak)` compares and divides in `double`.
      let limit := 1.0 / peak.toFloat
      (if rgain.toFloat > limit then limit else rgain.toFloat).toFloat32
    else rgain
  | none =>
    -- A NaN fallback counts as set; `-0.0` does not.
    if g.rgainFallback != 0 then (dbGain g.rgainFallback.toFloat).toFloat32 else 1.0

/-- The factor applied to every audio sample: the cube of the volume, times
the replaygain factor and the gain option, or 0 when muted. -/
-- mpv-cover: player/audio.c audio_get_gain paths=all hash=cfded804efbe
def audioGetGain (g : Gain) : Float32 :=
  let volume := g.softvolVolume.toFloat / 100.0
  let gain := (if volume > 0 then volume else 0).toFloat32
  let gain := (Float.pow gain.toFloat 3).toFloat32
  let gain := gain * computeReplaygain g
  let gain := (gain.toFloat * dbGain g.softvolGain.toFloat).toFloat32
  if g.softvolMute then 0.0 else gain

/-- The timestamp at the start of the audio written to the output so far, or
`nopts` without an output chain. -/
-- mpv-cover: player/audio.c written_audio_pts paths=all hash=3e61b9e64be4
def writtenAudioPts (c : Ctx) : Float :=
  match c.aoChain with
  | some chain => chain.lastOutPts
  | none => nopts

/-- The timestamp of the audio being heard now: the written one less what the
output still holds. `aoDelay` is what `ao_get_delay` in `audio/out/buffer.c`
returns, which the model does not cover yet. -/
-- mpv-cover: player/audio.c playing_audio_pts paths=all hash=406977672d2a
def playingAudioPts (aoDelay : Float) (c : Ctx) : Float :=
  let pts := writtenAudioPts c
  if pts == nopts || !c.ao then pts else pts - c.audioSpeed * aoDelay

end Mpv.Player.Audio
