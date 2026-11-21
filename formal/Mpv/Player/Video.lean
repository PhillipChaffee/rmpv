import Mpv.Player.Audio

/-!
A model of the A/V sync arithmetic in mpv's `player/video.c`: the drift
measurement and its correction, frame duration estimates, and the display-sync
speed search.

The float convention is the one in `Playloop.lean`. These functions share no
field with the playloop queries, so they have their own `Ctx`. They read the
audio clock through `written_audio_pts` and `playing_audio_pts`, so the context
holds an `Audio.Ctx` and the definitions call the audio model.
-/

namespace Mpv.Player.Video

open Mpv.Player.Playloop (nopts fclamp)
open Mpv.Player.Audio (writtenAudioPts playingAudioPts)

/-- `STATUS_PLAYING` in `enum playback_status`. -/
def statusPlaying : Int := 2

/-- `VS_DISP_RESAMPLE` in `enum video_sync`. -/
def vsDispResample : Int := 1

/-- `VS_DISP_TEMPO` in `enum video_sync`. -/
def vsDispTempo : Int := 4

/-- `MP_PTS_ROUNDING_TOLERANCE`: how far a Matroska timestamp, rounded to a
millisecond, can be from the true one. -/
def ptsRoundingTolerance : Float := 0.001 * 3 + 0.0001

/-- `MPMIN` on doubles: the first operand unless it is greater. -/
def fmin (a b : Float) : Float := if a > b then b else a

/-- The C library's `rint` in the default rounding mode: the nearest integer,
ties to even, with the sign of the argument. Lean has no `rint`, so this is
built from `floor`; the fidelity check compares it with libm's, except for the
sign of a zero result, which it compares nowhere. -/
def rint (x : Float) : Float :=
  let below := x.floor
  let rest := x - below
  let nearest :=
    if rest < 0.5 then below
    else if rest > 0.5 then below + 1
    else if (below / 2).floor == below / 2 then below
    else below + 1
  if nearest == 0 && x < 0 then -0.0 else nearest

/-- The `struct frame_info` fields the model reads. -/
structure Frame where
  /-- `duration`: the timestamp difference to the next frame. -/
  duration : Float
  /-- `approx_duration`: the duration with timestamp rounding smoothed out. -/
  approxDuration : Float

/-- The `struct vo_chain` fields the model reads. -/
structure VoChain where
  /-- `is_sparse`. -/
  isSparse : Bool
  /-- `filter->container_fps`. -/
  containerFps : Float

/-- The `struct MPContext` fields the model reads, with the option and output
fields reached through it. -/
structure Ctx where
  /-- `ao_chain`, `ao` and `audio_speed`. -/
  audio : Audio.Ctx
  /-- `audio_status`, an `enum playback_status`. -/
  audioStatus : Int
  /-- `video_status`. -/
  videoStatus : Int
  /-- `opts->audio_delay`. -/
  audioDelay : Float
  /-- `opts->default_max_pts_correction`, a `float`; negative means automatic. -/
  defaultMaxPtsCorrection : Float32
  /-- `delay`. -/
  delay : Float
  /-- `total_avsync_change`. -/
  totalAvsyncChange : Float
  /-- `display_sync_active`. -/
  displaySyncActive : Bool
  /-- `last_av_difference`. -/
  lastAvDifference : Float
  /-- `vo_chain`; `none` is `NULL`. -/
  voChain : Option VoChain
  /-- `video_pts`. -/
  videoPts : Float
  /-- `video_pts_unrounded`. -/
  videoPtsUnrounded : Float
  /-- `drop_message_shown`: the desync warning has been printed. -/
  dropMessageShown : Bool
  /-- `opts->playback_speed`. -/
  playbackSpeed : Float
  /-- `opts->sync_max_video_change`, in percent. -/
  syncMaxVideoChange : Float
  /-- `opts->sync_max_audio_change`, in percent. -/
  syncMaxAudioChange : Float
  /-- `opts->sync_max_factor`. -/
  syncMaxFactor : Int
  /-- `video_out->opts->video_sync`, an `enum video_sync`. -/
  videoSync : Int
  /-- `speed_factor_v`. -/
  speedFactorV : Float
  /-- `speed_factor_a`. -/
  speedFactorA : Float
  /-- `audio_drift_compensation`. -/
  audioDriftCompensation : Float
  /-- `avd_filtered`: the low-pass filtered A/V difference. -/
  avdFiltered : Float
  /-- `past_frames`, newest first; the size is `num_past_frames`. -/
  pastFrames : Array Frame
  /-- `next_frames[i]->pts`; the size is `num_next_frames`. -/
  nextFrames : Array Float

/-- Moves `delay` a step towards cancelling the difference between the written
audio position and the video timestamp `vPts`. The step is a tenth of the
difference, limited to the pts-correction option or to a share of
`frameTime`. -/
-- mpv-cover: player/video.c adjust_sync paths=all hash=a8c5411ec911
def adjustSync (c : Ctx) (vPts frameTime : Float) : Ctx :=
  if c.audioStatus ≠ statusPlaying then c else
  let aPts := writtenAudioPts c.audio + c.audioDelay - c.delay
  let avDelay := aPts - vPts
  let change := avDelay * 0.1
  let factor : Float := if avDelay.abs < 0.3 then 0.1 else 0.4
  let maxChange :=
    if c.defaultMaxPtsCorrection ≥ 0 then c.defaultMaxPtsCorrection.toFloat
    else frameTime * factor
  let change :=
    if change < -maxChange then -maxChange
    else if change > maxChange then maxChange
    else change
  { c with
    delay := c.delay + change
    totalAvsyncChange := if c.displaySyncActive then 0 else c.totalAvsyncChange + change }

/-- Records the A/V difference for a frame shown `offset` seconds from now: 0
unless both streams are playing, the video is not sparse and both positions
are known. A difference above half a second sets `dropMessageShown`, which is
where mpv prints its desync warning once. `aoDelay` is what `ao_get_delay`
returns. -/
-- mpv-cover: player/video.c update_av_diff paths=all hash=a667ca1f0c71
def updateAvDiff (aoDelay : Float) (c : Ctx) (offset : Float) : Ctx :=
  if c.audioStatus ≠ statusPlaying ∨ c.videoStatus ≠ statusPlaying then
    { c with lastAvDifference := 0 }
  else if c.voChain.any (·.isSparse) then { c with lastAvDifference := 0 }
  else
    let aPos := playingAudioPts aoDelay c.audio
    let difference :=
      if aPos != nopts && c.videoPts != nopts then
        aPos - c.videoPtsUnrounded + c.audioDelay + offset
      else 0
    { c with
      lastAvDifference := difference
      dropMessageShown := c.dropMessageShown || difference.abs > 0.5 }

/-- The `approx_duration` of the past frames that have a positive one. A NaN
duration is kept: only `dur <= 0` skips a frame. -/
def positiveDurations (c : Ctx) : List Float :=
  (c.pastFrames.toList.map (·.approxDuration)).filter fun dur => !(dur ≤ 0)

/-- The mean of `durations` summed front to back, or `otherwise` when there
are none. -/
def meanOr (durations : List Float) (otherwise : Float) : Float :=
  if durations.length > 0 then durations.foldl (· + ·) 0 / Float.ofNat durations.length
  else otherwise

/-- The mean smoothed duration of the past frames, or 0 when none has a
positive one. -/
-- mpv-cover: player/video.c calc_average_frame_duration paths=all hash=9f3d0785dd93
def calcAverageFrameDuration (c : Ctx) : Float :=
  meanOr (positiveDurations c) 0

/-- The speed change that fits `factor` frames into a whole number of vsyncs,
if it is within `maxChange` of 1. -/
def speedForFactor (ratio maxChange : Float) (factor : Nat) : Option Float :=
  let scale := ratio * Float.ofNat factor / rint (ratio * Float.ofNat factor)
  if (scale - 1).abs ≤ maxChange then some scale else none

/-- A speed factor that makes `factor` frames of length `frame` last a whole
number of vsyncs of length `vsync`, for the smallest `factor` from 1 to
`maxFactor` that needs a change of at most `maxChange`. -1 when there is
none. -/
-- mpv-cover: player/video.c calc_best_speed paths=all hash=90e00f8065d4
def calcBestSpeed (vsync frame maxChange : Float) (maxFactor : Int) : Float :=
  let ratio := frame / vsync
  ((List.range maxFactor.toNat).findSome? fun n => speedForFactor ratio maxChange (n + 1)).getD (-1)

/-- The mean of `calcBestSpeed` over the past frames it finds a speed for, or
1 when it finds none. -/
-- mpv-cover: player/video.c find_best_speed paths=all hash=fa70c59fa73f
def findBestSpeed (c : Ctx) (vsync : Float) : Float :=
  let speeds := (positiveDurations c).filterMap fun dur =>
    let best := calcBestSpeed vsync (dur / c.playbackSpeed) (c.syncMaxVideoChange / 100)
      c.syncMaxFactor
    if best ≤ 0 then none else some best
  meanOr speeds 1

/-- Whether the audio output plays a compressed stream as is, so that it
cannot be resampled. `aoFormat` is the format `ao_get_format` in
`audio/out/ao.c` reports and `isPcm` is `af_fmt_is_pcm` in `audio/format.c`;
the model covers neither yet. -/
-- mpv-cover: player/video.c using_spdif_passthrough paths=all hash=44b0e075f6af
def usingSpdifPassthrough (aoFormat : Int) (isPcm : Int → Bool) (c : Ctx) : Bool :=
  match c.audio.aoChain with
  | some chain => if chain.ao then !isPcm aoFormat else false
  | none => false

/-- Sets the audio speed factor for display sync after a frame shown for
`duration` seconds. Outside the two resampling modes, or while audio is not
playing, audio follows video and the filter is cleared. Otherwise the A/V
difference is low-pass filtered and the compensation moves, at a limited rate,
towards the value that lets the filtered difference decay.

C reads `video_out->opts` without a test, so the model describes mpv only
while `video_out` is not `NULL`. Its one caller, `handle_display_sync_frame`,
reads the same field first, so it cannot make the call with `NULL`. -/
-- mpv-cover: player/video.c adjust_audio_drift_compensation paths=all hash=4bf2a35e76f8
def adjustAudioDriftCompensation (c : Ctx) (duration : Float) : Ctx :=
  if (c.videoSync ≠ vsDispResample ∧ c.videoSync ≠ vsDispTempo) ∨
      c.audioStatus ≠ statusPlaying then
    { c with speedFactorA := c.speedFactorV, audioDriftCompensation := 0, avdFiltered := 0 }
  else
    let maxCorrect := c.syncMaxAudioChange / 100
    let other := c.playbackSpeed * c.speedFactorV
    let comp := 1.0 + c.audioDriftCompensation
    let alpha := duration / (1.0 + duration)
    let filtered := c.avdFiltered + alpha * (c.lastAvDifference - c.avdFiltered)
    let pDelta := fclamp (-filtered / (10.0 * other)) (-maxCorrect) maxCorrect
    let slew := fmin pDelta.abs (maxCorrect / 10.0)
    let targetComp := fclamp (1.0 + pDelta) (1 - maxCorrect) (1 + maxCorrect)
    let newComp := comp + fclamp (targetComp - comp) (-slew) slew
    { c with
      avdFiltered := filtered
      audioDriftCompensation := newComp - 1.0
      speedFactorA := newComp * c.speedFactorV }

/-- The duration `calculate_frame_duration` starts from: the difference of the
next two timestamps when both are known and in order, else `demuxDuration`. -/
def nextFrameDuration (c : Ctx) (demuxDuration : Float) : Float :=
  match c.nextFrames[0]?, c.nextFrames[1]? with
  | some pts0, some pts1 =>
    if pts0 != nopts && pts1 != nopts && pts1 ≥ pts0 then pts1 - pts0 else demuxDuration
  | _, _ => demuxDuration

/-- The durations of the frames before the newest, up to the first that is
not positive or is `ptsRoundingTolerance` or more from `duration`. -/
def similarDurations (c : Ctx) (duration : Float) : List Float :=
  ((c.pastFrames.toList.drop 1).map (·.duration)).takeWhile fun dur =>
    !(dur ≤ 0 || (dur - duration).abs ≥ ptsRoundingTolerance)

/-- Fills in `duration` and `approx_duration` of the newest past frame. The
duration is the gap to the frame after next, else the container's frame time.
The smoothed duration is the mean over the recent frames that agree with it,
or the container's frame time when that fits them.

C asserts `num_past_frames >= 1 && num_next_frames >= 1` and reads
`vo_chain->filter` without a test, so the model describes mpv only while both
arrays are non-empty and `vo_chain` is not `NULL`. Its one caller,
`write_video`, returns on entry when `vo_chain` is `NULL`. -/
-- mpv-cover: player/video.c calculate_frame_duration paths=all unobservable=20,21,22,23 hash=183893b1e7ad
def calculateFrameDuration (c : Ctx) : Ctx :=
  match c.voChain with
  | none => c
  | some vo =>
    let demuxDuration := if vo.containerFps > 0 then 1.0 / vo.containerFps else -1
    let duration := nextFrameDuration c demuxDuration
    let similar := similarDurations c duration
    let total := similar.foldl (· + ·) 0
    let approx := meanOr similar duration
    let approx :=
      if demuxDuration > 0 ∧ (duration - demuxDuration).abs < ptsRoundingTolerance ∧
          (total - demuxDuration * Float.ofNat similar.length).abs < ptsRoundingTolerance ∧
          (similar.length ≥ 16 ∨ (similar.length : Int) ≥ c.pastFrames.size - 4) then
        demuxDuration
      else approx
    { c with
      pastFrames := c.pastFrames.modify 0 fun frame =>
        { frame with duration, approxDuration := approx } }

end Mpv.Player.Video
