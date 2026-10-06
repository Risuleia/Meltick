Texture2D DigitAtlas : register(t1);
Texture2D AmPmAtlas : register(t2);
SamplerState InputSampler : register(s0);

cbuffer BgParams : register(b0)
{
    float2 resolution;
    float  time;
    float  _pad;

    float4 card0;
    float4 card1;

    float4 roll0_a;
    float4 roll0_b;

    float4 roll1_a;
    float4 roll1_b;

    float  ampm_visible;
    float  is_pm;
    float2 _ampm_pad;
};

static const float ATLAS_W = 7200.0;
static const float ATLAS_H = 1792.0;
static const float CELL_W  = 720.0;
static const float CELL_H  = 1792.0;

static const float AMPM_ATLAS_W = 2880.0;
static const float AMPM_ATLAS_H = 1792.0;
static const float AMPM_CELL_W  = 960.0;
static const float AMPM_CELL_H  = 1792.0;


float sampleDigitSDF(
    float2 px,
    float2 digitCenter,
    float digitHeight,
    int digit
)
{
    float digitWidth =
        digitHeight * (CELL_W / CELL_H);

    float2 q =
        (px - digitCenter) /
        float2(digitWidth, digitHeight);

    float2 localUv = q + 0.5;

    const float PAD_X = 4.0 / CELL_W;
    const float PAD_Y = 4.0 / CELL_H;

    localUv = clamp(
        localUv,
        float2(PAD_X, PAD_Y),
        float2(1.0 - PAD_X, 1.0 - PAD_Y)
    );

    float cellX =
        (digit + localUv.x) / 10.0;

    float2 uv =
        float2(cellX, localUv.y);

    float encoded =
        DigitAtlas.SampleLevel(
            InputSampler,
            uv,
            0
        ).r;

    float sdf =
        (encoded - 0.5) * 256.0;

    sdf *= digitHeight / CELL_H;

    return sdf;
}

float sampleAmPmSDF(
    float2 px,
    float2 glyphCenter,
    float glyphHeight,
    int glyph
)
{
    float glyphWidth =
        glyphHeight * (AMPM_CELL_W / AMPM_CELL_H);

    float2 q =
        (px - glyphCenter) /
        float2(glyphWidth, glyphHeight);

    float2 localUv = q + 0.5;

    const float PAD_X = 4.0 / AMPM_CELL_W;
    const float PAD_Y = 4.0 / AMPM_CELL_H;

    localUv = clamp(
        localUv,
        float2(PAD_X, PAD_Y),
        float2(1.0 - PAD_X, 1.0 - PAD_Y)
    );

    float cellX =
        (glyph + localUv.x) / 3.0;

    float2 uv =
        float2(cellX, localUv.y);

    float encoded =
        AmPmAtlas.SampleLevel(
            InputSampler,
            uv,
            0
        ).r;

    float sdf =
        (encoded - 0.5) * 256.0;

    sdf *= glyphHeight / AMPM_CELL_H;

    return sdf;
}

float sdfToAlpha(float sdf)
{
    float aa = max(fwidth(sdf), 0.75);

    return smoothstep(
        -aa,
        aa,
        sdf
    );
}

float rollSquash(float progress)
{
    float center =
        1.0 - abs(progress * 2.0 - 1.0);

    center =
        center * center *
        (3.0 - 2.0 * center);

    return lerp(
        1.0,
        0.64,
        center
    );
}

float renderRollingDigit(
    float2 px,
    float2 center,
    float digitHeight,
    float currentDigit,
    float nextDigit,
    float progress
)
{
    float squash =
        rollSquash(progress);

    float2 currentCenter =
        center +
        float2(
            0.0,
            -digitHeight * progress
        );

    float2 currentPx =
        currentCenter +
        (px - currentCenter) /
        float2(
            sqrt(squash),
            squash
        );

    float currentSdf =
        sampleDigitSDF(
            currentPx,
            currentCenter,
            digitHeight,
            (int)currentDigit
        );

    float2 nextCenter =
        center +
        float2(
            0.0,
            digitHeight * (1.0 - progress)
        );

    float2 nextPx =
        nextCenter +
        (px - nextCenter) /
        float2(
            sqrt(squash),
            squash
        );

    float nextSdf =
        sampleDigitSDF(
            nextPx,
            nextCenter,
            digitHeight,
            (int)nextDigit
        );

    float d =
        max(
            currentSdf,
            nextSdf
        );

    float currentAlpha = sdfToAlpha(currentSdf);
    float nextAlpha = sdfToAlpha(nextSdf);

    return max(
        currentAlpha,
        nextAlpha
    );
}


float renderTwoDigits(
    float2 px,
    float4 card,
    float4 rollA,
    float4 rollB
)
{
    float2 center = card.xy;

    float digitHeight = card.w;

    float digitWidth =
        digitHeight * (CELL_W / CELL_H);

    float totalWidth =
        digitWidth * 2.0;

    float2 leftCenter =
        center +
        float2(
            -totalWidth * 0.5 + digitWidth * 0.5,
            0.0
        );

    float2 rightCenter =
        center +
        float2(
            totalWidth * 0.5 - digitWidth * 0.5,
            0.0
        );

    float d0;

    if (rollA.w > 0.5)
    {
        d0 =
            renderRollingDigit(
                px,
                leftCenter,
                digitHeight,
                rollA.x,
                rollA.y,
                rollA.z
            );
    }
    else
    {
        d0 =
            sampleDigitSDF(
                px,
                leftCenter,
                digitHeight,
                (int)rollA.x
            );

        d0 = sdfToAlpha(d0);
    }

    float d1;

    if (rollB.w > 0.5)
    {
        d1 =
            renderRollingDigit(
                px,
                rightCenter,
                digitHeight,
                rollB.x,
                rollB.y,
                rollB.z
            );
    }
    else
    {
        d1 =
            sampleDigitSDF(
                px,
                rightCenter,
                digitHeight,
                (int)rollB.x
            );

        d1 = sdfToAlpha(d1);
    }

    return max(d0, d1);
}

float renderAmPm(
    float2 px,
    float4 card
)
{
    if (ampm_visible <= 0.5)
        return 0.0;

    float2 center = card.xy;

    float glyphHeight =
        card.w * 0.11;

    float glyphWidth =
        glyphHeight *
        (AMPM_CELL_W / AMPM_CELL_H);

    float totalWidth =
        glyphWidth * 2.0;

    float padding =
        card.w * 0.06;

    float2 indicatorCenter =
        center +
        float2(
            -card.z * 0.5 +
                padding +
                totalWidth * 0.5,

            card.w * 0.5 -
                padding -
                glyphHeight * 0.5
        );

    float2 leftCenter =
        indicatorCenter +
        float2(
            -totalWidth * 0.25,
            0.0
        );

    float2 rightCenter =
        indicatorCenter +
        float2(
            totalWidth * 0.25,
            0.0
        );

    int firstGlyph =
        is_pm > 0.5 ? 2 : 0;

    float leftSdf =
        sampleAmPmSDF(
            px,
            leftCenter,
            glyphHeight,
            firstGlyph
        );

    float rightSdf =
        sampleAmPmSDF(
            px,
            rightCenter,
            glyphHeight,
            1
        );

    float leftAlpha =
        sdfToAlpha(leftSdf);

    float rightAlpha =
        sdfToAlpha(rightSdf);

    return max(
        leftAlpha,
        rightAlpha
    );
}

float cardSDF(
    float2 px,
    float4 card
)
{
    float2 center = card.xy;
    float2 size   = card.zw;

    float radius =
        min(size.x, size.y) * 0.12;

    float2 p =
        px - center;

    float2 q =
        abs(p) -
        (size * 0.5 - radius);

    return
        length(max(q, 0.0)) +
        min(max(q.x, q.y), 0.0) -
        radius;
}