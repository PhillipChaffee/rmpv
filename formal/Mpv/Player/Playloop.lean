/-!
A model of the playback time and position queries in mpv's `player/playloop.c`.

A C `double` is a Lean `Float`, which is the same IEEE-754 binary64 type. Each
definition performs the C function's operations in the C function's order, with
IEEE arithmetic as the C source reads. Comparisons are written the way the C
macros write them, because `MPMAX` and `MPCLAMP` are not symmetric when an
operand is NaN.

Results agree with mpv bit for bit, with one exception: the sign of a zero.
mpv is built with `-fno-signed-zeros`, which lets the compiler treat `-0.0` and
`+0.0` as the same value, so mpv's source does not decide which of the two a
zero result is. Where a definition or a theorem here gives a result of `0`, the
model's value is `+0.0`, and what is claimed of mpv is a zero of either sign.
The fidelity check does not compare the sign of a zero.
-/

namespace Mpv.Player.Playloop

/-- `MP_NOPTS_VALUE`, which is `-0x1p+63`: the time that means "unknown". -/
def nopts : Float := Float.ofBits 0xC3E0000000000000

/-- `MPMAX` on doubles: the second operand unless the first is greater. -/
def fmax (a b : Float) : Float := if a > b then a else b

/-- `MPCLAMP` on doubles. -/
def fclamp (a low high : Float) : Float :=
  if a < low then low else if a > high then high else a

/-- The `struct demuxer` fields the model reads. -/
structure Demuxer where
  /-- `duration`, negative when unknown. -/
  duration : Float
  /-- `start_time`. -/
  startTime : Float
  /-- `filesize`, an `int64_t`. -/
  filesize : Int
  /-- `filepos`, an `int64_t`. -/
  filepos : Int

/-- The `struct MPContext` fields the model reads, with the option and demuxer
fields reached through it. -/
structure Ctx where
  /-- `demuxer`; `none` is `NULL`. -/
  demuxer : Option Demuxer
  /-- `opts->rebase_start_time`. -/
  rebaseStartTime : Bool
  /-- `opts->play_frames`. -/
  playFrames : Int
  /-- `playback_pts`. -/
  playbackPts : Float
  /-- `play_dir`. -/
  playDir : Int
  /-- `last_seek_pts`. -/
  lastSeekPts : Float
  /-- `max_frames`. -/
  maxFrames : Int
  /-- `chapters[i].pts`; the size is `num_chapters`. -/
  chapters : Array Float
  /-- `last_chapter_seek`. -/
  lastChapterSeek : Int
  /-- `last_chapter_flag`. -/
  lastChapterFlag : Bool
  /-- `cache_buffer`. -/
  cacheBuffer : Int

/-- The duration, or `nopts` when there is no demuxer or the duration is unknown. -/
-- mpv-cover: player/playloop.c get_time_length paths=all hash=ddbffa1985d3
def getTimeLength (c : Ctx) : Float :=
  match c.demuxer with
  | some d => if d.duration ≥ 0 then d.duration else nopts
  | none => nopts

/-- The approximate time of the first frame played in direction `dir`. -/
-- mpv-cover: player/playloop.c get_start_time paths=all hash=8220dc2e47a2
def getStartTime (c : Ctx) (dir : Int) : Float :=
  match c.demuxer with
  | none => 0
  | some d =>
    -- `0 + start_time` is kept as the C source has it. In IEEE arithmetic it
    -- turns a start time of `-0.0` into `+0.0`. mpv's real build does not: under
    -- `-fno-signed-zeros` the compiler drops the addition and returns `-0.0`.
    -- The two differ only in the sign of a zero, which the model does not claim.
    let res : Float := if !c.rebaseStartTime then 0 + d.startTime else 0
    if dir < 0 then res + fmax d.duration 0 else res

/-- The playback position, or the last seek target while no frame has been
shown since the seek. -/
-- mpv-cover: player/playloop.c get_current_time paths=all hash=586d4b9d00b3
def getCurrentTime (c : Ctx) : Float :=
  match c.demuxer with
  | none => nopts
  | some _ =>
    if c.playbackPts != nopts then c.playbackPts * Float.ofInt c.playDir
    else c.lastSeekPts

/-- `getCurrentTime` for display: a seek target is clamped to the file, and no
known time is negative. -/
-- mpv-cover: player/playloop.c get_playback_time paths=all hash=176d510d0023
def getPlaybackTime (c : Ctx) : Float :=
  let cur := getCurrentTime c
  let cur :=
    if cur != nopts && c.playbackPts == nopts then
      let length := getTimeLength c
      if length ≥ 0 then fclamp cur 0 length else cur
    else cur
  if cur != nopts ∧ cur < 0 then 0 else cur

/-- The playback position as a ratio from 0 to 1, or -1 when unknown.
`playStart` and `playEnd` are `get_play_start_pts` and `get_play_end_pts` from
`player/misc.c`, which the model does not cover yet. -/
-- mpv-cover: player/playloop.c get_current_pos_ratio paths=all hash=23a4685954fe
def getCurrentPosRatio (playStart playEnd : Ctx → Float) (c : Ctx) (useRange : Bool) :
    Float :=
  match c.demuxer with
  | none => -1
  | some d =>
    let len := getTimeLength c
    let (start, len) :=
      if useRange then
        let startpos := playStart c
        let endpos := playEnd c
        let endpos := if endpos > fmax 0 len then fmax 0 len else endpos
        let endpos := if endpos < startpos then startpos else endpos
        (startpos, endpos - startpos)
      else (0, len)
    let pos := getCurrentTime c
    let ret : Float := if len > 0 then fclamp ((pos - start) / len) 0 1 else -1
    let ret :=
      if ret < 0 then
        if d.filesize > 0 ∧ d.filepos ≥ 0 then
          fclamp (Float.ofInt d.filepos / Float.ofInt d.filesize) 0 1
        else ret
      else ret
    if useRange ∧ c.playFrames > 0 then
      fmax ret (1.0 - Float.ofInt c.maxFrames / Float.ofInt c.playFrames)
    else ret

/-- How many leading chapters start at or before `time`: where the C loop stops. -/
def chaptersStarted (c : Ctx) (time : Float) : Nat :=
  c.chapters.findIdx fun pts => time < pts

/-- The index of the current chapter: -2 without chapters, -1 before the first.
A chapter seek that is still pending overrides the position. -/
-- mpv-cover: player/playloop.c get_current_chapter paths=all hash=ef6243bbf31d
def getCurrentChapter (c : Ctx) : Int :=
  if c.chapters.size = 0 then -2 else
  let i : Int := chaptersStarted c (getCurrentTime c)
  if c.lastChapterFlag then c.lastChapterSeek else max c.lastChapterSeek (i - 1)

/-- The start of a chapter; chapter -1 starts at 0. `nopts` when there is no
such chapter. -/
-- mpv-cover: player/playloop.c chapter_start_time paths=all hash=5ec6a0930aaf
def chapterStartTime (c : Ctx) (chapter : Int) : Float :=
  if chapter = -1 then 0
  else if h : 0 ≤ chapter ∧ chapter < c.chapters.size then
    c.chapters[chapter.toNat]'(by omega)
  else nopts

-- mpv-cover: player/playloop.c get_chapter_count paths=all hash=f6287e9c9fba
def getChapterCount (c : Ctx) : Int := c.chapters.size

/-- The cache fill percentage, or -1 without a demuxer. -/
-- mpv-cover: player/playloop.c get_cache_buffering_percentage paths=all hash=d3eb3542f162
def getCacheBufferingPercentage (c : Ctx) : Int :=
  if c.demuxer.isSome then c.cacheBuffer else -1

end Mpv.Player.Playloop
