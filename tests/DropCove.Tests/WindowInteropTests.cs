using DropCove.Native;

namespace DropCove.Tests;

[TestClass]
public sealed class WindowInteropTests
{
    [TestMethod]
    [DataRow(420, 96u, 420)]
    [DataRow(420, 144u, 630)]
    [DataRow(700, 192u, 1400)]
    [DataRow(1, 144u, 2)]
    public void ScaleLogicalPixels_UsesEffectiveMonitorDpi(int logicalPixels, uint dpi, int expectedPhysicalPixels)
    {
        var scaled = WindowInterop.ScaleLogicalPixels(logicalPixels, dpi);

        Assert.AreEqual(expectedPhysicalPixels, scaled);
    }

    [TestMethod]
    [DataRow(420, 96u, 420)]
    [DataRow(630, 144u, 420)]
    [DataRow(1400, 192u, 700)]
    [DataRow(2, 144u, 1)]
    [DataRow(180, 96u, 180)]
    [DataRow(270, 144u, 180)]
    [DataRow(360, 192u, 180)]
    public void UnscalePhysicalPixels_InvertsScalingDeterministically(int physicalPixels, uint dpi, int expectedLogicalPixels)
    {
        var unscaled = WindowInterop.UnscalePhysicalPixels(physicalPixels, dpi);

        Assert.AreEqual(expectedLogicalPixels, unscaled);
    }

    [TestMethod]
    [DataRow(96u)]
    [DataRow(144u)]
    [DataRow(192u)]
    public void ScaleAndUnscale_RoundTripPreservesLogicalDimension(uint dpi)
    {
        const int logicalDimension = WindowInterop.MinimumShelfDimension;
        var physical = WindowInterop.ScaleLogicalPixels(logicalDimension, dpi);
        var restoredLogical = WindowInterop.UnscalePhysicalPixels(physical, dpi);

        Assert.AreEqual(logicalDimension, restoredLogical);
    }

    [TestMethod]
    [DataRow(100, 100, 200, 200, 100, 100, WindowResizeDirection.TopLeft, 13)]
    [DataRow(100, 100, 200, 200, 107, 107, WindowResizeDirection.TopLeft, 13)]
    [DataRow(100, 100, 200, 200, 199, 100, WindowResizeDirection.TopRight, 14)]
    [DataRow(100, 100, 200, 200, 192, 107, WindowResizeDirection.TopRight, 14)]
    [DataRow(100, 100, 200, 200, 100, 199, WindowResizeDirection.BottomLeft, 16)]
    [DataRow(100, 100, 200, 200, 107, 192, WindowResizeDirection.BottomLeft, 16)]
    [DataRow(100, 100, 200, 200, 199, 199, WindowResizeDirection.BottomRight, 17)]
    [DataRow(100, 100, 200, 200, 192, 192, WindowResizeDirection.BottomRight, 17)]
    [DataRow(100, 100, 200, 200, 100, 150, WindowResizeDirection.Left, 10)]
    [DataRow(100, 100, 200, 200, 107, 150, WindowResizeDirection.Left, 10)]
    [DataRow(100, 100, 200, 200, 199, 150, WindowResizeDirection.Right, 11)]
    [DataRow(100, 100, 200, 200, 192, 150, WindowResizeDirection.Right, 11)]
    [DataRow(100, 100, 200, 200, 150, 100, WindowResizeDirection.Top, 12)]
    [DataRow(100, 100, 200, 200, 150, 107, WindowResizeDirection.Top, 12)]
    [DataRow(100, 100, 200, 200, 150, 199, WindowResizeDirection.Bottom, 15)]
    [DataRow(100, 100, 200, 200, 150, 192, WindowResizeDirection.Bottom, 15)]
    [DataRow(100, 100, 200, 200, 150, 150, WindowResizeDirection.None, 1)]
    [DataRow(100, 100, 200, 200, 108, 108, WindowResizeDirection.None, 1)]
    [DataRow(100, 100, 200, 200, 191, 191, WindowResizeDirection.None, 1)]
    public void HitTestBorder_DetectsAllEightEdgesAndClient_WithStandardBorder(
        int left,
        int top,
        int right,
        int bottom,
        int testX,
        int testY,
        WindowResizeDirection expectedDirection,
        int expectedHitCode)
    {
        var bounds = new WindowBounds(left, top, right, bottom);
        var result = WindowInterop.HitTestBorder(bounds, testX, testY, WindowInterop.DefaultResizeBorderThickness);

        Assert.AreEqual(expectedDirection, result.Direction);
        Assert.AreEqual((nint)expectedHitCode, result.HitCode);
        Assert.AreEqual(expectedDirection != WindowResizeDirection.None, result.IsResizeBorder);
    }

    [TestMethod]
    [DataRow(-500, -300, -200, -100, -500, -300, WindowResizeDirection.TopLeft, 13)]
    [DataRow(-500, -300, -200, -100, -201, -300, WindowResizeDirection.TopRight, 14)]
    [DataRow(-500, -300, -200, -100, -500, -101, WindowResizeDirection.BottomLeft, 16)]
    [DataRow(-500, -300, -200, -100, -201, -101, WindowResizeDirection.BottomRight, 17)]
    [DataRow(-500, -300, -200, -100, -493, -200, WindowResizeDirection.Left, 10)]
    [DataRow(-500, -300, -200, -100, -208, -200, WindowResizeDirection.Right, 11)]
    [DataRow(-500, -300, -200, -100, -350, -293, WindowResizeDirection.Top, 12)]
    [DataRow(-500, -300, -200, -100, -350, -108, WindowResizeDirection.Bottom, 15)]
    [DataRow(-500, -300, -200, -100, -350, -200, WindowResizeDirection.None, 1)]
    [DataRow(-500, -300, -200, -100, -501, -200, WindowResizeDirection.None, 0)]
    [DataRow(-500, -300, -200, -100, -200, -200, WindowResizeDirection.None, 0)]
    [DataRow(-500, -300, -200, -100, -350, -301, WindowResizeDirection.None, 0)]
    [DataRow(-500, -300, -200, -100, -350, -100, WindowResizeDirection.None, 0)]
    public void HitTestBorder_SupportsNegativeScreenCoordinates(
        int left,
        int top,
        int right,
        int bottom,
        int testX,
        int testY,
        WindowResizeDirection expectedDirection,
        int expectedHitCode)
    {
        var bounds = new WindowBounds(left, top, right, bottom);
        var result = WindowInterop.HitTestBorder(bounds, testX, testY, WindowInterop.DefaultResizeBorderThickness);

        Assert.AreEqual(expectedDirection, result.Direction);
        Assert.AreEqual((nint)expectedHitCode, result.HitCode);
    }

    [TestMethod]
    public void ConstrainResizeBounds_ResizingRightEdge_AnchorsLeftEdge()
    {
        var workArea = new WindowBounds(0, 0, 1920, 1080);
        var candidate = new WindowBounds(100, 100, 150, 300); // requested width 50 < min 180

        var constrained = WindowInterop.ConstrainResizeBounds(
            candidate,
            WindowResizeDirection.Right,
            180,
            180,
            96u,
            workArea);

        Assert.AreEqual(100, constrained.Left);
        Assert.AreEqual(280, constrained.Right);
        Assert.AreEqual(180, constrained.Width);
        Assert.AreEqual(100, constrained.Top);
        Assert.AreEqual(300, constrained.Bottom);
    }

    [TestMethod]
    public void ConstrainResizeBounds_ResizingLeftEdge_AnchorsRightEdge()
    {
        var workArea = new WindowBounds(0, 0, 1920, 1080);
        var candidate = new WindowBounds(350, 100, 400, 300); // requested width 50 < min 180

        var constrained = WindowInterop.ConstrainResizeBounds(
            candidate,
            WindowResizeDirection.Left,
            180,
            180,
            96u,
            workArea);

        Assert.AreEqual(400, constrained.Right);
        Assert.AreEqual(220, constrained.Left);
        Assert.AreEqual(180, constrained.Width);
    }

    [TestMethod]
    public void ConstrainResizeBounds_ResizingBottomEdge_AnchorsTopEdge()
    {
        var workArea = new WindowBounds(0, 0, 1920, 1080);
        var candidate = new WindowBounds(100, 100, 300, 150); // requested height 50 < min 180

        var constrained = WindowInterop.ConstrainResizeBounds(
            candidate,
            WindowResizeDirection.Bottom,
            180,
            180,
            96u,
            workArea);

        Assert.AreEqual(100, constrained.Top);
        Assert.AreEqual(280, constrained.Bottom);
        Assert.AreEqual(180, constrained.Height);
    }

    [TestMethod]
    public void ConstrainResizeBounds_ResizingTopEdge_AnchorsBottomEdge()
    {
        var workArea = new WindowBounds(0, 0, 1920, 1080);
        var candidate = new WindowBounds(100, 350, 300, 400); // requested height 50 < min 180

        var constrained = WindowInterop.ConstrainResizeBounds(
            candidate,
            WindowResizeDirection.Top,
            180,
            180,
            96u,
            workArea);

        Assert.AreEqual(400, constrained.Bottom);
        Assert.AreEqual(220, constrained.Top);
        Assert.AreEqual(180, constrained.Height);
    }

    [TestMethod]
    public void ConstrainResizeBounds_ResizingCorner_EnforcesIndependentAxesWithoutAspectLock()
    {
        var workArea = new WindowBounds(0, 0, 1920, 1080);
        var candidate = new WindowBounds(100, 100, 500, 200); // width 400 (>= 180), height 100 (< 180)

        var constrained = WindowInterop.ConstrainResizeBounds(
            candidate,
            WindowResizeDirection.BottomRight,
            180,
            180,
            96u,
            workArea);

        Assert.AreEqual(100, constrained.Left);
        Assert.AreEqual(500, constrained.Right);
        Assert.AreEqual(400, constrained.Width);
        Assert.AreEqual(100, constrained.Top);
        Assert.AreEqual(280, constrained.Bottom);
        Assert.AreEqual(180, constrained.Height);
    }

    [TestMethod]
    public void ConstrainResizeBounds_ResizingTopLeft_AnchorsBottomRightCorner()
    {
        var workArea = new WindowBounds(0, 0, 1920, 1080);
        var candidate = new WindowBounds(350, 350, 400, 400); // width 50, height 50

        var constrained = WindowInterop.ConstrainResizeBounds(
            candidate,
            WindowResizeDirection.TopLeft,
            180,
            180,
            96u,
            workArea);

        Assert.AreEqual(400, constrained.Right);
        Assert.AreEqual(400, constrained.Bottom);
        Assert.AreEqual(220, constrained.Left);
        Assert.AreEqual(220, constrained.Top);
        Assert.AreEqual(180, constrained.Width);
        Assert.AreEqual(180, constrained.Height);
    }

    [TestMethod]
    public void ConstrainResizeBounds_ScalesMinimumDimensionWithHighDpi()
    {
        var workArea = new WindowBounds(0, 0, 3840, 2160);
        var candidate = new WindowBounds(100, 100, 250, 250); // width 150 < 180 * 1.5 = 270 physical

        var constrained = WindowInterop.ConstrainResizeBounds(
            candidate,
            WindowResizeDirection.Right,
            180,
            180,
            144u,
            workArea);

        Assert.AreEqual(100, constrained.Left);
        Assert.AreEqual(370, constrained.Right);
        Assert.AreEqual(270, constrained.Width);
    }

    [TestMethod]
    public void ConstrainResizeBounds_ClampsCandidateCrossingWorkAreaBoundaries()
    {
        var workArea = new WindowBounds(100, 100, 1000, 800);
        var candidate = new WindowBounds(50, 50, 400, 400);

        var constrained = WindowInterop.ConstrainResizeBounds(
            candidate,
            WindowResizeDirection.TopLeft,
            180,
            180,
            96u,
            workArea);

        Assert.AreEqual(100, constrained.Left);
        Assert.AreEqual(100, constrained.Top);
    }

    [TestMethod]
    public void ConstrainResizeBounds_WorkAreaSmallerThanMinimum_PrioritizesFittingInWorkArea()
    {
        // Spec §95: If a work area is smaller than the minimum, keeping the full window reachable takes precedence over the minimum.
        var tightWorkArea = new WindowBounds(50, 50, 210, 210); // 160 x 160 physical work area
        var candidate = new WindowBounds(50, 50, 300, 300);

        var constrained = WindowInterop.ConstrainResizeBounds(
            candidate,
            WindowResizeDirection.BottomRight,
            180,
            180,
            96u,
            tightWorkArea);

        Assert.AreEqual(50, constrained.Left);
        Assert.AreEqual(50, constrained.Top);
        Assert.AreEqual(210, constrained.Right);
        Assert.AreEqual(210, constrained.Bottom);
        Assert.AreEqual(160, constrained.Width);
        Assert.AreEqual(160, constrained.Height);
    }

    [TestMethod]
    public void ConstrainResizeBounds_NegativeScreenCoordinates_ClampsWithinWorkArea()
    {
        var negativeWorkArea = new WindowBounds(-1920, -1080, 0, 0);
        var candidate = new WindowBounds(-2000, -500, -1700, -200);

        var constrained = WindowInterop.ConstrainResizeBounds(
            candidate,
            WindowResizeDirection.Left,
            180,
            180,
            96u,
            negativeWorkArea);

        Assert.IsLessThanOrEqualTo(negativeWorkArea.Right, constrained.Right);
        Assert.AreEqual(-1920, constrained.Left);
    }

    [TestMethod]
    public void PlaceAnchoredPopup_FirstFittingRight_PlacesFlushRightTopAligned()
    {
        var workArea = new WindowBounds(0, 0, 1920, 1080);
        var anchor = new WindowBounds(200, 200, 400, 300); // 200x100 anchor

        var popup = WindowInterop.PlaceAnchoredPopup(
            anchor,
            workArea,
            requestedLogicalWidth: 350,
            requestedLogicalHeight: 400,
            dpi: 96u);

        Assert.AreEqual(400, popup.Left);
        Assert.AreEqual(200, popup.Top);
        Assert.AreEqual(750, popup.Right);
        Assert.AreEqual(600, popup.Bottom);
        Assert.AreEqual(350, popup.Width);
        Assert.AreEqual(400, popup.Height);
    }

    [TestMethod]
    public void PlaceAnchoredPopup_RightBlocked_PlacesFlushLeftTopAligned()
    {
        var workArea = new WindowBounds(0, 0, 1920, 1080);
        var anchor = new WindowBounds(1600, 200, 1800, 300); // anchor right at 1800. At 350 width, right candidate ends at 2150 > safeRight (1904)

        var popup = WindowInterop.PlaceAnchoredPopup(
            anchor,
            workArea,
            requestedLogicalWidth: 350,
            requestedLogicalHeight: 400,
            dpi: 96u);

        Assert.AreEqual(1250, popup.Left);
        Assert.AreEqual(200, popup.Top);
        Assert.AreEqual(1600, popup.Right);
        Assert.AreEqual(600, popup.Bottom);
        Assert.AreEqual(350, popup.Width);
        Assert.AreEqual(400, popup.Height);
    }

    [TestMethod]
    public void PlaceAnchoredPopup_RightAndLeftBlocked_PlacesBelowLeftAligned()
    {
        var workArea = new WindowBounds(0, 0, 500, 1080);
        // WorkArea 500 wide. safeLeft: 16, safeRight: 484.
        // Place anchor spanning width: left: 100, right: 400.
        // Right candidate: 400..750 (exceeds safeRight 484)
        // Left candidate: -250..100 (below safeLeft 16)
        // Below candidate: Left=100, Right=450 <= 484, Top=300, Bottom=700 <= 1064 -> fits!
        var anchor = new WindowBounds(100, 200, 400, 300);

        var popup = WindowInterop.PlaceAnchoredPopup(
            anchor,
            workArea,
            requestedLogicalWidth: 350,
            requestedLogicalHeight: 400,
            dpi: 96u);

        Assert.AreEqual(100, popup.Left);
        Assert.AreEqual(300, popup.Top);
        Assert.AreEqual(450, popup.Right);
        Assert.AreEqual(700, popup.Bottom);
        Assert.AreEqual(350, popup.Width);
        Assert.AreEqual(400, popup.Height);
    }

    [TestMethod]
    public void PlaceAnchoredPopup_RightLeftBelowBlocked_PlacesAboveLeftAligned()
    {
        var workArea = new WindowBounds(0, 0, 500, 1080);
        // WorkArea 500 wide, 1080 high. safeBottom: 1064.
        // Place anchor near bottom: Top=700, Bottom=900.
        // Right candidate: 400..750 (blocked)
        // Left candidate: -250..100 (blocked)
        // Below candidate: Top=900, Bottom=1300 > 1064 (blocked)
        // Above candidate: Left=100, Right=450 <= 484, Top=300 >= 16, Bottom=700 <= 1064 -> fits!
        var anchor = new WindowBounds(100, 700, 400, 900);

        var popup = WindowInterop.PlaceAnchoredPopup(
            anchor,
            workArea,
            requestedLogicalWidth: 350,
            requestedLogicalHeight: 400,
            dpi: 96u);

        Assert.AreEqual(100, popup.Left);
        Assert.AreEqual(300, popup.Top);
        Assert.AreEqual(450, popup.Right);
        Assert.AreEqual(700, popup.Bottom);
        Assert.AreEqual(350, popup.Width);
        Assert.AreEqual(400, popup.Height);
    }

    [TestMethod]
    public void PlaceAnchoredPopup_AllBlocked_ClampsRightCandidateInsideWorkAreaMargin()
    {
        var workArea = new WindowBounds(0, 0, 500, 500);
        // safe bounds: [16, 16, 484, 484]
        // Anchor occupies center: Left=80, Top=80, Right=420, Bottom=420.
        // Target size: 350x350
        // Right: [420..770, 80..430] -> right 770 > 484 (blocked)
        // Left: [70..420, 80..430] -> left 70 is ok, but wait: 80 - 350 = -270 < 16 (blocked)
        // Below: [80..430, 420..770] -> bottom 770 > 484 (blocked)
        // Above: [80..430, -270..80] -> top -270 < 16 (blocked)
        // All 4 fail! Fallback clamps Right candidate:
        // clampedMinX = 16, clampedMaxX = 500 - 16 - 350 = 134.
        // RightCandidate Left = 420. Clamped to 134!
        // clampedMinY = 16, clampedMaxY = 500 - 16 - 350 = 134.
        // RightCandidate Top = 80. Inside [16, 134] -> remains 80.
        var anchor = new WindowBounds(80, 80, 420, 420);

        var popup = WindowInterop.PlaceAnchoredPopup(
            anchor,
            workArea,
            requestedLogicalWidth: 350,
            requestedLogicalHeight: 350,
            dpi: 96u);

        Assert.AreEqual(134, popup.Left);
        Assert.AreEqual(80, popup.Top);
        Assert.AreEqual(484, popup.Right);
        Assert.AreEqual(430, popup.Bottom);
        Assert.AreEqual(350, popup.Width);
        Assert.AreEqual(350, popup.Height);
    }

    [TestMethod]
    [DataRow(100, 0, 320, 1)]      // Below minimums: clamps to 320x1
    [DataRow(320, 1, 320, 1)]      // Exact minimums
    [DataRow(400, 300, 400, 300)]  // Within range
    [DataRow(480, 480, 480, 480)]  // Exact maximums
    [DataRow(600, 700, 480, 480)]  // Above maximums: clamps to 480x480
    public void PlaceAnchoredPopup_LogicalSizeLimits_ClampsWidthAndHeight(
        int requestedW,
        int requestedH,
        int expectedW,
        int expectedH)
    {
        var workArea = new WindowBounds(0, 0, 1920, 1080);
        var anchor = new WindowBounds(100, 100, 200, 200);

        var popup = WindowInterop.PlaceAnchoredPopup(
            anchor,
            workArea,
            requestedLogicalWidth: requestedW,
            requestedLogicalHeight: requestedH,
            dpi: 96u);

        Assert.AreEqual(expectedW, popup.Width);
        Assert.AreEqual(expectedH, popup.Height);
    }

    [TestMethod]
    [DataRow(96u, 320, 480, 16, 400, 300)]     // 100% scale
    [DataRow(144u, 480, 720, 24, 600, 450)]    // 150% scale
    [DataRow(192u, 640, 960, 32, 800, 600)]    // 200% scale
    public void PlaceAnchoredPopup_SupportsDifferentDpis(
        uint dpi,
        int expectedMinWidth,
        int expectedMaxWidth,
        int expectedMargin,
        int expectedScaledW,
        int expectedScaledH)
    {
        var workArea = new WindowBounds(0, 0, 3840, 2160);
        var anchor = new WindowBounds(200, 200, 400, 400);

        // Request 400x300 logical
        var popup = WindowInterop.PlaceAnchoredPopup(
            anchor,
            workArea,
            requestedLogicalWidth: 400,
            requestedLogicalHeight: 300,
            dpi: dpi);

        Assert.AreEqual(expectedScaledW, popup.Width);
        Assert.AreEqual(expectedScaledH, popup.Height);
        Assert.AreEqual(anchor.Right, popup.Left);
        Assert.AreEqual(anchor.Top, popup.Top);
        Assert.IsLessThanOrEqualTo(workArea.Right - expectedMargin, popup.Right);
        Assert.IsLessThanOrEqualTo(workArea.Bottom - expectedMargin, popup.Bottom);
    }

    [TestMethod]
    public void PlaceAnchoredPopup_NegativeScreenCoordinates_PlacesAndClampsProperly()
    {
        // Secondary monitor placed to the left of primary: [-1920, 0, 0, 1080]
        var negativeWorkArea = new WindowBounds(-1920, 0, 0, 1080);
        var anchor = new WindowBounds(-500, 200, -300, 300); // 200x100

        var popup = WindowInterop.PlaceAnchoredPopup(
            anchor,
            negativeWorkArea,
            requestedLogicalWidth: 350,
            requestedLogicalHeight: 400,
            dpi: 96u);

        // Right candidate: Left = -300, Right = 50. SafeRight = 0 - 16 = -16. Right (50) > -16 -> blocked!
        // Left candidate: Left = -500 - 350 = -850, Right = -500.
        // SafeLeft = -1920 + 16 = -1904. -850 >= -1904, -500 <= -16 -> fits!
        Assert.AreEqual(-850, popup.Left);
        Assert.AreEqual(200, popup.Top);
        Assert.AreEqual(-500, popup.Right);
        Assert.AreEqual(600, popup.Bottom);
    }

    [TestMethod]
    public void PlaceAnchoredPopup_TinyWorkArea_OverridesMinimumsAndPreservesAvailableMargin()
    {
        // WorkArea is smaller than the minimum popup width (320) and height
        var tinyWorkArea = new WindowBounds(100, 100, 300, 250); // Width = 200, Height = 150
        var anchor = new WindowBounds(150, 120, 250, 180);

        var popup = WindowInterop.PlaceAnchoredPopup(
            anchor,
            tinyWorkArea,
            requestedLogicalWidth: 400,
            requestedLogicalHeight: 400,
            dpi: 96u);

        Assert.AreEqual(new WindowBounds(116, 116, 284, 234), popup);
    }

    [TestMethod]
    public void PlaceAnchoredPopup_SizeLimitExceedsInnerWorkArea_ShrinksBeforeDroppingMargin()
    {
        var popup = WindowInterop.PlaceAnchoredPopup(
            new WindowBounds(200, 200, 300, 300), new WindowBounds(0, 0, 500, 500), 480, 480, 96);

        Assert.AreEqual(new WindowBounds(16, 16, 484, 484), popup);
    }

    [TestMethod]
    public void PlaceAnchoredPopup_OnePixelWorkArea_DropsImpossibleMargin()
    {
        var workArea = new WindowBounds(-100, -100, -99, -99);

        var popup = WindowInterop.PlaceAnchoredPopup(new WindowBounds(0, 0, 100, 100), workArea, 480, 480, 192);

        Assert.AreEqual(workArea, popup);
    }
}
