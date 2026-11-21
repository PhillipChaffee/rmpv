import Mpv.Player.Audio
import Oracle.Script

/-!
The part of the script the `audio` and `video` oracles share: the audio clock,
which `player/audio.c` computes and `player/video.c` reads.
-/

open Mpv.Player.Audio
open Oracle.Script

namespace Oracle.AvSync

/-- The audio clock's inputs: the context, and what the stand-in for
`ao_get_delay` returns. -/
structure Clock where
  ctx : Ctx
  aoDelay : Float

def Clock.empty : Clock :=
  { ctx := { aoChain := none, ao := false, audioSpeed := 0 }, aoDelay := 0 }

def Clock.show (k : Clock) : String :=
  let chain := match k.ctx.aoChain with
    | some chain => s!"{showFloat chain.lastOutPts},{bit chain.ao}"
    | none => "none"
  s!"ac={chain} ao={bit k.ctx.ao} as={showFloat k.ctx.audioSpeed} dly={showFloat k.aoDelay}"

/-- The operations that set the clock's inputs; `none` for any other. -/
def Clock.set (k : Clock) (op : String) (args : List String) : Option Clock :=
  match op, args with
  | "nochain", [] => some { k with ctx.aoChain := none }
  | "chain", [pts, ao] =>
    some { k with ctx.aoChain := some { lastOutPts := float pts, ao := int ao ≠ 0 } }
  | "ao", [ao, speed, delay] =>
    some { ctx := { k.ctx with ao := int ao ≠ 0, audioSpeed := float speed },
           aoDelay := float delay }
  | _, _ => none

end Oracle.AvSync
