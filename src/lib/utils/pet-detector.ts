// Window is 160x160
// Cat container is 120x120 centered → offset 20px
// Panda head region (approximate in window px):
//   x: 30 to 130
//   y: 10 to 70 (top portion of panda)

export function isCursorOnHead(
  cursorX: number,
  cursorY: number,
  winX: number,
  winY: number,
  _winWidth: number,
  _winHeight: number
): boolean {
  const relX = cursorX - winX;
  const relY = cursorY - winY;

  return relX >= 30 && relX <= 130 && relY >= 10 && relY <= 70;
}
