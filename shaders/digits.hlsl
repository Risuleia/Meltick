#include "digit.hlsli"

float4 main(float4 pos : SV_POSITION) : SV_TARGET
{
    float2 px = pos.xy;

    float card0Dist =
        cardSDF(px, card0);

    float card1Dist =
        cardSDF(px, card1);

    float cardMask =
        min(
            card0Dist,
            card1Dist
        );

    if (cardMask > 0.0)
        discard;

    float d0 =
        renderTwoDigits(
            pos.xy,
            card0,
            roll0_a,
            roll0_b
        );

    float d1 =
        renderTwoDigits(
            pos.xy,
            card1,
            roll1_a,
            roll1_b
        );

    float digit =
        max(d0, d1);

    float ampm =
            renderAmPm(
                pos.xy,
                card0
            );

    digit = max(
        digit,
        ampm
    );

    return float4(
        1.0,
        1.0,
        1.0,
        digit * 0.72
    );
}