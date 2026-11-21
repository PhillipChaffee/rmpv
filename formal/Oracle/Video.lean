import Mpv.Player.Video
import Oracle.AvSync

/-!
Runs the video model on a script of operations, one per line on stdin, and
prints the result and the state after each. `fidelity/` feeds the same script to
mpv's real `video.c` and compares the output.
-/

open Mpv.Player.Video
open Oracle.Script
open Oracle.AvSync (Clock)

namespace Oracle.Video

/-- The state the script builds: the context, and what the stand-ins for
`ao_get_delay`, `ao_get_format` and `af_fmt_is_pcm` answer. The context's
audio fields are the clock's. -/
structure World where
  ctx : Ctx
  aoDelay : Float
  /-- The format the audio output reports. -/
  aoFormat : Int
  /-- The one format that counts as PCM. -/
  pcmFormat : Int

def empty : World :=
  { ctx := { audio := Clock.empty.ctx, audioStatus := 0, videoStatus := 0, audioDelay := 0,
             defaultMaxPtsCorrection := 0, delay := 0, totalAvsyncChange := 0,
             displaySyncActive := false, lastAvDifference := 0, voChain := none, videoPts := 0,
             videoPtsUnrounded := 0, dropMessageShown := false, playbackSpeed := 0,
             syncMaxVideoChange := 0, syncMaxAudioChange := 0, syncMaxFactor := 0,
             videoSync := 0, speedFactorV := 0, speedFactorA := 0, audioDriftCompensation := 0,
             avdFiltered := 0, pastFrames := #[], nextFrames := #[] },
    aoDelay := 0, aoFormat := 0, pcmFormat := 0 }

def World.clock (w : World) : Clock := { ctx := w.ctx.audio, aoDelay := w.aoDelay }

def showState (w : World) : String :=
  let c := w.ctx
  let vo := match c.voChain with
    | some vo => s!"{bit vo.isSparse},{showFloat vo.containerFps}"
    | none => "none"
  let past := ",".intercalate
    (c.pastFrames.toList.map fun f => s!"{showFloat f.duration}:{showFloat f.approxDuration}")
  let next := ",".intercalate (c.nextFrames.toList.map showFloat)
  s!"{w.clock.show} st={c.audioStatus},{c.videoStatus} adel={showFloat c.audioDelay} \
     mpc={showFloat32 c.defaultMaxPtsCorrection} d={showFloat c.delay} \
     tac={showFloat c.totalAvsyncChange} dsa={bit c.displaySyncActive} \
     lav={showFloat c.lastAvDifference} vc={vo} vpts={showFloat c.videoPts} \
     vptsu={showFloat c.videoPtsUnrounded} dms={bit c.dropMessageShown} \
     spd={showFloat c.playbackSpeed} smv={showFloat c.syncMaxVideoChange} \
     sma={showFloat c.syncMaxAudioChange} smf={c.syncMaxFactor} vs={c.videoSync} \
     sfv={showFloat c.speedFactorV} sfa={showFloat c.speedFactorA} \
     adc={showFloat c.audioDriftCompensation} avdf={showFloat c.avdFiltered} \
     fmt={w.aoFormat},{w.pcmFormat} pf={past} nf={next}"

/-- The frames of a `past` operation: a duration and a smoothed duration each. -/
def frames : List String → List Frame
  | duration :: approx :: rest =>
    { duration := float duration, approxDuration := float approx } :: frames rest
  | _ => []

/-- One script operation: its printed result and the state after it. -/
def step (w : World) (op : String) (args : List String) : String × World :=
  let c := w.ctx
  let set (ctx : Ctx) : String × World := ("ok", { w with ctx })
  let time (x : Float) : String × World := (showFloat x, w)
  match w.clock.set op args with
  | some clock => ("ok", { w with ctx.audio := clock.ctx, aoDelay := clock.aoDelay })
  | none =>
  match op, args with
  | "reset", [] => ("ok", empty)
  | "status", [audio, video] => set { c with audioStatus := int audio, videoStatus := int video }
  | "sync", [audioDelay, correction, delay, total, active] =>
    set { c with audioDelay := float audioDelay, defaultMaxPtsCorrection := float32 correction,
                 delay := float delay, totalAvsyncChange := float total,
                 displaySyncActive := int active ≠ 0 }
  | "avstate", [difference, pts, unrounded, shown] =>
    set { c with lastAvDifference := float difference, videoPts := float pts,
                 videoPtsUnrounded := float unrounded, dropMessageShown := int shown ≠ 0 }
  | "novo", [] => set { c with voChain := none }
  | "vo", [sparse, fps] =>
    set { c with voChain := some { isSparse := int sparse ≠ 0, containerFps := float fps } }
  | "speed", [speed, video, audio, factor] =>
    set { c with playbackSpeed := float speed, syncMaxVideoChange := float video,
                 syncMaxAudioChange := float audio, syncMaxFactor := int factor }
  | "dsync", [mode, factorV, factorA, compensation, filtered] =>
    set { c with videoSync := int mode, speedFactorV := float factorV,
                 speedFactorA := float factorA, audioDriftCompensation := float compensation,
                 avdFiltered := float filtered }
  | "fmt", [format, pcm] => ("ok", { w with aoFormat := int format, pcmFormat := int pcm })
  | "past", _ :: words => set { c with pastFrames := (frames words).toArray }
  | "next", _ :: times => set { c with nextFrames := (times.map float).toArray }
  | "consts", [] =>
    (s!"{statusPlaying},{vsDispResample},{vsDispTempo},{showFloat ptsRoundingTolerance},\
        {showFloat Mpv.Player.Playloop.nopts}", w)
  | "rint", [x] => time (rint (float x))
  | "adjsync", [pts, frameTime] => set (adjustSync c (float pts) (float frameTime))
  | "avdiff", [offset] => set (updateAvDiff w.aoDelay c (float offset))
  | "avgdur", [] => time (calcAverageFrameDuration c)
  | "bestspeed", [vsync, frame, maxChange, maxFactor] =>
    time (calcBestSpeed (float vsync) (float frame) (float maxChange) (int maxFactor))
  | "findspeed", [vsync] => time (findBestSpeed c (float vsync))
  | "spdif", [] => (bit (usingSpdifPassthrough w.aoFormat (· == w.pcmFormat) c), w)
  | "drift", [duration] => set (adjustAudioDriftCompensation c (float duration))
  | "framedur", [] => set (calculateFrameDuration c)
  | _, _ => ("bad", w)

end Oracle.Video

def main : IO Unit := do
  Oracle.Script.loop Oracle.Video.step Oracle.Video.showState (← IO.getStdin) Oracle.Video.empty
