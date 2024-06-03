#ifndef TONE_H
#define TONE_H

// (参考) https://docs.arduino.cc/built-in-examples/digital/toneMelody/

// 以下の理由から低音域は定義しない
// - B0, B1 が Arudino の define マクロと衝突してエラーになる
// - 低音域はブザーの音色がそもそも良くないため
namespace Note {

constexpr unsigned int C2 = 65;
constexpr unsigned int D2 = 73;
constexpr unsigned int E2 = 82;
constexpr unsigned int F2 = 87;
constexpr unsigned int G2 = 98;
constexpr unsigned int A2 = 110;
constexpr unsigned int B2 = 123;
constexpr unsigned int C3 = 131;
constexpr unsigned int D3 = 147;
constexpr unsigned int E3 = 165;
constexpr unsigned int F3 = 175;
constexpr unsigned int G3 = 196;
constexpr unsigned int A3 = 220;
constexpr unsigned int B3 = 247;
constexpr unsigned int C4 = 262;
constexpr unsigned int D4 = 294;
constexpr unsigned int E4 = 330;
constexpr unsigned int F4 = 349;
constexpr unsigned int G4 = 392;
constexpr unsigned int A4 = 440;
constexpr unsigned int B4 = 494;
constexpr unsigned int C5 = 523;
constexpr unsigned int D5 = 587;
constexpr unsigned int E5 = 659;
constexpr unsigned int F5 = 698;
constexpr unsigned int G5 = 784;
constexpr unsigned int A5 = 880;
constexpr unsigned int B5 = 988;
constexpr unsigned int C6 = 1047;
constexpr unsigned int D6 = 1175;
constexpr unsigned int E6 = 1319;
constexpr unsigned int F6 = 1397;
constexpr unsigned int G6 = 1568;
constexpr unsigned int A6 = 1760;
constexpr unsigned int B6 = 1976;
constexpr unsigned int C7 = 2093;
constexpr unsigned int D7 = 2349;
constexpr unsigned int E7 = 2637;
constexpr unsigned int F7 = 2794;
constexpr unsigned int G7 = 3136;
constexpr unsigned int A7 = 3520;
constexpr unsigned int B7 = 3951;
constexpr unsigned int C8 = 4186;
constexpr unsigned int D8 = 4699;

}

#endif // TONE_H
