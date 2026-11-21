/-!
How values cross an oracle's script and output. A float crosses as the hex
digits of its IEEE-754 bit pattern, 16 for a `double` and 8 for a `float`, so
mpv and the model start from the identical value and the comparison is exact.
-/

namespace Oracle.Script

def hexDigit (c : Char) : Nat :=
  if c.isDigit then c.toNat - '0'.toNat else c.toNat - 'a'.toNat + 10

def hex (word : String) : Nat :=
  word.foldl (fun n c => 16 * n + hexDigit c) 0

def showHex (width n : Nat) : String :=
  let digits := Nat.toDigits 16 n
  String.ofList (List.replicate (width - digits.length) '0' ++ digits)

def float (word : String) : Float := Float.ofBits (hex word).toUInt64

def showFloat (x : Float) : String := showHex 16 x.toBits.toNat

def float32 (word : String) : Float32 := Float32.ofBits (hex word).toUInt32

def showFloat32 (x : Float32) : String := showHex 8 x.toBits.toNat

def int (word : String) : Int := word.toInt?.getD 0

def bit (b : Bool) : String := if b then "1" else "0"

/-- Runs `step` on each line of stdin and prints its result and the state
after it. -/
partial def loop {World : Type} (step : World → String → List String → String × World)
    (showState : World → String) (stdin : IO.FS.Stream) (w : World) : IO Unit := do
  let line ← stdin.getLine
  if line.isEmpty then return
  match line.trimAscii.toString.splitOn " " with
  | op :: words =>
    let (result, next) := step w op words
    IO.println s!"{result} | {showState next}"
    loop step showState stdin next
  | [] => loop step showState stdin w

end Oracle.Script
