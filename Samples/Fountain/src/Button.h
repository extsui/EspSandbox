#ifndef BUTTON_H
#define BUTTON_H

// 割り込みを使用する場合に有効にすること
// WasXxx() 系の関数が割り込み排他で呼び出されるようになる
#define BUTTON_CONFIG_USE_INTERRUPT (1)

/**
 * Arudino 用ボタン管理クラス
 * 
 * 定期的に Update() を呼び出すことで、チャタリングを除去した上で
 * ボタン押下イベントを取得できる。
 */
class Button
{
public:
    Button();
    void Initialize(int pinNumber, bool isPositive, bool isPullUp);

    // ------------------------------
    //      イベント発生の仕様
    // ------------------------------
    //                   _____
    // _________________|     |_____
    //                        ^
    //              SingleClickEvent
    //           _____________
    // _________|             |_____
    //                        ^
    //               LongClickEvent
    //       _____       _____
    // _____|     |_____|     |_____
    //            ^           ^
    //     SingleClickEvent   |
    //              DoubleClickEvent
    //
    // DoubleClickEvent 成立の制約
    // - 一方が LongClick の場合は不成立とする

    // イベント発生後に 1 回だけ true を返す
    // 上記以外は false を返す
    bool WasSingleClicked();
    bool WasLongClicked();
    bool WasDoubleClicked();

    // 全イベントのクリア
    void ClearEvents();

    // 状態更新
    void Update();

private:
    bool Read();

private:
    // 設定
    int m_PinNumber;
    bool m_IsInitialized;
    bool m_IsPositive;

    // サンプリング基準
    uint32_t m_NextSampleTime;

    // チャタリング除去用 (3回分)
    // - [ false --> true  --> true  ] で状態を true (押下) に遷移
    // - [ true  --> false --> false ] で状態を false (離し) に遷移
    bool m_History[3];

    // チャタリング除去済みのボタン状態 (true: 押下 / false: 離し)
    // イベント生成にはこれを使用する
    bool m_State;

    // 特殊イベント用
    uint32_t m_PressStartTime;
    uint32_t m_LastSingleClickedTime;

    // 押下イベント有無
    bool m_WasSingleClicked;
    bool m_WasLongClicked;
    bool m_WasDoubleClicked;
};

#endif /* BUTTON_H */
