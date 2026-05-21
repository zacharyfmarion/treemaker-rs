import origami.crease_pattern.CustomLineTypes;
import origami.Epsilon;
import origami.crease_pattern.FoldLineSet;
import origami.crease_pattern.LineSegmentSet;
import origami.crease_pattern.OritaCalc;
import origami.crease_pattern.element.Circle;
import origami.crease_pattern.element.LineColor;
import origami.crease_pattern.element.LineSegment;
import origami.crease_pattern.element.Point;
import origami.crease_pattern.element.Polygon;
import origami.crease_pattern.element.StraightLine;
import origami.crease_pattern.worker.foldlineset.BranchTrim;
import origami.crease_pattern.worker.foldlineset.Fix1;
import origami.crease_pattern.worker.foldlineset.Fix2;
import origami.crease_pattern.worker.foldlineset.OrganizeCircles;
import origami.crease_pattern.worker.linesegmentset.IntersectDivide;
import origami.folding.util.SortingBox;

import oriedita.editor.databinding.GridModel;
import oriedita.editor.export.DxfExporter;
import oriedita.editor.export.ObjImporter;
import oriedita.editor.export.OrhExporter;
import oriedita.editor.export.OrhImporter;
import oriedita.editor.save.Save;
import oriedita.editor.save.SaveProvider;

import java.awt.Color;
import java.io.File;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.util.HashSet;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;

public class OrieditaGeometryOracle {
    public static void main(String[] args) throws Exception {
        if (args.length < 1) {
            usage("missing command");
        }

        switch (args[0]) {
            case "intersection" -> intersection(args);
            case "intersect-divide" -> intersectDivide(args);
            case "intersect-divide-pair" -> intersectDividePair(args);
            case "foldline-divide-new-lines" -> foldLineDivideNewLines(args);
            case "foldline-divide-fast" -> foldLineDivideFast(args);
            case "foldline-delete-inside" -> foldLineDeleteInside(args);
            case "foldline-delete-line-vertex" -> foldLineDeleteLineVertex(args);
            case "foldline-delete-lines" -> foldLineDeleteLines(args);
            case "foldline-branch-trim" -> foldLineBranchTrim(args);
            case "foldline-del-v" -> foldLineDelV(args);
            case "foldline-del-v-cc" -> foldLineDelVCc(args);
            case "foldline-del-v-pair" -> foldLineDelVPair(args);
            case "foldline-del-v-all" -> foldLineDelVAll(args);
            case "foldline-del-v-all-cc" -> foldLineDelVAllCc(args);
            case "foldline-fix1" -> foldLineFix1(args);
            case "foldline-fix2" -> foldLineFix2(args);
            case "foldline-set-color" -> foldLineSetColor(args);
            case "foldline-change-type" -> foldLineChangeType(args);
            case "foldline-make-color" -> foldLineMakeColor(args);
            case "foldline-make-aux" -> foldLineMakeAux(args);
            case "foldline-change-mv" -> foldLineChangeMv(args);
            case "foldline-advance-type" -> foldLineAdvanceType(args);
            case "foldline-alternate-mv" -> foldLineAlternateMv(args);
            case "foldline-alternate-mv-crossing" -> foldLineAlternateMvCrossing(args);
            case "foldline-select-all" -> foldLineSelectAll(args);
            case "foldline-select-indices" -> foldLineSelectIndices(args);
            case "foldline-select-box" -> foldLineSelectBox(args);
            case "foldline-select-polygon" -> foldLineSelectPolygon(args);
            case "foldline-select-lx" -> foldLineSelectLx(args);
            case "foldline-select-connected" -> foldLineSelectConnected(args);
            case "foldline-delete-selected" -> foldLineDeleteSelected(args);
            case "foldline-replace-type" -> foldLineReplaceType(args);
            case "foldline-delete-type" -> foldLineDeleteType(args);
            case "foldline-translate" -> foldLineTranslate(args);
            case "foldline-transform-selected" -> foldLineTransformSelected(args);
            case "foldline-transform-selected-4p" -> foldLineTransformSelected4p(args);
            case "foldline-extend-to-intersection" -> foldLineExtendToIntersection(args);
            case "foldline-draw-crease" -> foldLineDrawCrease(args);
            case "foldline-draw-symmetric" -> foldLineDrawSymmetric(args);
            case "foldline-draw-point" -> foldLineDrawPoint(args);
            case "foldline-circle-draw" -> foldLineCircleDraw(args);
            case "foldline-circle-draw-free" -> foldLineCircleDrawFree(args);
            case "foldline-circle-three-point" -> foldLineCircleThreePoint(args);
            case "foldline-circle-separate" -> foldLineCircleSeparate(args);
            case "foldline-circle-concentric" -> foldLineCircleConcentric(args);
            case "foldline-circle-concentric-select" -> foldLineCircleConcentricSelect(args);
            case "foldline-circle-concentric-two" -> foldLineCircleConcentricTwo(args);
            case "foldline-circle-invert-circle" -> foldLineCircleInvertCircle(args);
            case "foldline-circle-invert-line" -> foldLineCircleInvertLine(args);
            case "foldline-circle-organize" -> foldLineCircleOrganize(args);
            case "foldline-circle-change-color" -> foldLineCircleChangeColor(args);
            case "foldline-circle-tangent-point" -> foldLineCircleTangentPoint(args);
            case "foldline-circle-tangent-two" -> foldLineCircleTangentTwo(args);
            case "foldline-divide-count" -> foldLineDivideCount(args);
            case "foldline-divide-ratio" -> foldLineDivideRatio(args);
            case "measure-length" -> measureLength(args);
            case "measure-angle" -> measureAngle(args);
            case "custom-line-type" -> customLineType(args);
            case "orh-import-summary" -> orhImportSummary(args);
            case "orh-export-fixture" -> orhExportFixture(args);
            case "obj-import-summary" -> objImportSummary(args);
            case "dxf-export-fixture" -> dxfExportFixture(args);
            default -> usage("unknown command: " + args[0]);
        }
    }

    private static void customLineType(String[] args) {
        if (args.length != 3) {
            usage("custom-line-type expects custom type number and line color number");
        }

        CustomLineTypes customLineType = CustomLineTypes.from(Integer.parseInt(args[1]));
        LineColor lineColor = LineColor.fromNumber(Integer.parseInt(args[2]));

        System.out.println(customLineType.getNumber()
                + ","
                + customLineType.getNumberForLineColor()
                + ","
                + customLineType.getLineColor().getNumber()
                + ","
                + customLineType.matches(lineColor));
    }

    private static void intersection(String[] args) {
        if (args.length != 11) {
            usage("intersection expects mode, precision, and eight coordinates");
        }

        boolean sweet = switch (args[1]) {
            case "strict" -> false;
            case "sweet" -> true;
            default -> throw new IllegalArgumentException("unknown intersection mode: " + args[1]);
        };

        String precisionArg = args[2];
        LineSegment s1 = new LineSegment(
                parse(args[3]),
                parse(args[4]),
                parse(args[5]),
                parse(args[6]));
        LineSegment s2 = new LineSegment(
                parse(args[7]),
                parse(args[8]),
                parse(args[9]),
                parse(args[10]));
        LineSegment.Intersection result;
        if (precisionArg.equals("default")) {
            result = sweet
                    ? OritaCalc.determineLineSegmentIntersectionSweet(s1, s2)
                    : OritaCalc.determineLineSegmentIntersection(s1, s2);
        } else {
            double precision = parse(precisionArg);
            result = sweet
                    ? OritaCalc.determineLineSegmentIntersectionSweet(s1, s2, precision, precision)
                    : OritaCalc.determineLineSegmentIntersection(s1, s2, precision);
        }

        System.out.println(result.getState());
    }

    private static void intersectDivide(String[] args) throws Exception {
        if (args.length < 2) {
            usage("intersect-divide expects a segment count and segment payload");
        }

        int count = Integer.parseInt(args[1]);
        LineSegmentSet set = lineSegmentSet(args, 2, count);
        IntersectDivide.apply(set);
        printLineSegmentSet(set);
    }

    private static void intersectDividePair(String[] args) throws Exception {
        if (args.length < 4) {
            usage("intersect-divide-pair expects i, j, a segment count, and segment payload");
        }

        int i = Integer.parseInt(args[1]);
        int j = Integer.parseInt(args[2]);
        int count = Integer.parseInt(args[3]);
        LineSegmentSet set = lineSegmentSet(args, 4, count);
        Method method = IntersectDivide.class.getDeclaredMethod(
                "intersect_divide",
                LineSegmentSet.class,
                int.class,
                int.class);
        method.setAccessible(true);

        int added = (Integer) method.invoke(null, set, i, j);
        System.out.println("added|" + added);
        printLineSegmentSet(set);
    }

    private static void foldLineDivideNewLines(String[] args) {
        if (args.length < 4) {
            usage("foldline-divide-new-lines expects originalEnd, addedEnd, count, and segment payload");
        }

        int originalEnd = Integer.parseInt(args[1]);
        int addedEnd = Integer.parseInt(args[2]);
        int count = Integer.parseInt(args[3]);
        FoldLineSet set = foldLineSet(args, 4, count);
        set.divideLineSegmentWithNewLines(originalEnd, addedEnd);
        printFoldLineSet(set);
    }

    private static void foldLineDivideFast(String[] args) {
        if (args.length < 4) {
            usage("foldline-divide-fast expects i, j, count, and segment payload");
        }

        int i = Integer.parseInt(args[1]);
        int j = Integer.parseInt(args[2]);
        int count = Integer.parseInt(args[3]);
        FoldLineSet set = foldLineSet(args, 4, count);
        Set<Integer> toDelete = new HashSet<>();
        LineSegment.Intersection intersection = set.divideIntersectionsFast(i + 1, j + 1, toDelete);
        System.out.println("intersection|" + intersection.getState());
        printFoldLineSetDeleteSet(toDelete);
        printFoldLineSet(set);
    }

    private static void foldLineDeleteInside(String[] args) {
        if (args.length < 8) {
            usage("foldline-delete-inside expects mode, selection segment, count, and segment payload");
        }

        String mode = args[1];
        LineSegment selection = new LineSegment(
                new Point(parse(args[2]), parse(args[3])),
                new Point(parse(args[4]), parse(args[5])),
                LineColor.fromNumber(Integer.parseInt(args[6])));
        int count = Integer.parseInt(args[7]);
        FoldLineSet set = foldLineSet(args, 8, count);
        boolean deleted = set.deleteInsideLine(selection, mode);
        System.out.println("deleted|" + deleted);
        printFoldLineSet(set);
    }

    private static void foldLineDeleteLineVertex(String[] args) {
        if (args.length < 3) {
            usage("foldline-delete-line-vertex expects index, count, and segment payload");
        }

        int index = Integer.parseInt(args[1]);
        int count = Integer.parseInt(args[2]);
        FoldLineSet set = foldLineSet(args, 3, count);
        boolean deleted = false;
        if (index >= 0 && index < set.getTotal()) {
            set.deleteLineSegment_vertex(set.get(index + 1));
            deleted = true;
        }
        System.out.println("deleted|" + deleted);
        printFoldLineSet(set);
    }

    private static void foldLineDeleteLines(String[] args) {
        if (args.length < 4) {
            usage("foldline-delete-lines expects comma-separated indices, count, and segment payload");
        }

        int count = Integer.parseInt(args[2]);
        FoldLineSet set = foldLineSet(args, 3, count);
        List<LineSegment> lines = selectedFoldLines(set, args[1]);
        int deleted = 0;
        for (LineSegment line : lines) {
            set.deleteLine(line);
            deleted++;
        }
        System.out.println("deleted|" + deleted);
        printFoldLineSet(set);
    }

    private static void foldLineBranchTrim(String[] args) {
        if (args.length < 2) {
            usage("foldline-branch-trim expects a segment count and segment payload");
        }

        int count = Integer.parseInt(args[1]);
        FoldLineSet set = foldLineSet(args, 2, count);
        BranchTrim.apply(set);
        printFoldLineSet(set);
    }

    private static void foldLineDelV(String[] args) {
        if (args.length < 6) {
            usage("foldline-del-v expects point, snap radius, vertex radius, count, and segment payload");
        }

        Point point = new Point(parse(args[1]), parse(args[2]));
        double snapRadius = parse(args[3]);
        double vertexRadius = parse(args[4]);
        int count = Integer.parseInt(args[5]);
        FoldLineSet set = foldLineSet(args, 6, count);
        boolean result = set.del_V(point, snapRadius, vertexRadius);
        System.out.println("result|" + result);
        printFoldLineSet(set);
    }

    private static void foldLineDelVCc(String[] args) {
        if (args.length < 6) {
            usage("foldline-del-v-cc expects point, snap radius, vertex radius, count, and segment payload");
        }

        Point point = new Point(parse(args[1]), parse(args[2]));
        double snapRadius = parse(args[3]);
        double vertexRadius = parse(args[4]);
        int count = Integer.parseInt(args[5]);
        FoldLineSet set = foldLineSet(args, 6, count);
        boolean result = set.del_V_cc(point, snapRadius, vertexRadius);
        System.out.println("result|" + result);
        printFoldLineSet(set);
    }

    private static void foldLineDelVPair(String[] args) {
        if (args.length < 4) {
            usage("foldline-del-v-pair expects i, j, count, and segment payload");
        }

        int i = Integer.parseInt(args[1]);
        int j = Integer.parseInt(args[2]);
        int count = Integer.parseInt(args[3]);
        FoldLineSet set = foldLineSet(args, 4, count);
        LineSegment result = set.del_V(set.get(i + 1), set.get(j + 1));
        if (result == null) {
            System.out.println("result|null");
        } else {
            System.out.println("result|"
                    + result.determineAX() + "|"
                    + result.determineAY() + "|"
                    + result.determineBX() + "|"
                    + result.determineBY() + "|"
                    + result.getColor().getNumber());
        }
        printFoldLineSet(set);
    }

    private static void foldLineDelVAll(String[] args) throws Exception {
        if (args.length < 2) {
            usage("foldline-del-v-all expects count and segment payload");
        }

        int count = Integer.parseInt(args[1]);
        FoldLineSet set = foldLineSet(args, 2, count);
        set.del_V_all();
        printFoldLineSet(set);
    }

    private static void foldLineDelVAllCc(String[] args) throws Exception {
        if (args.length < 2) {
            usage("foldline-del-v-all-cc expects count and segment payload");
        }

        int count = Integer.parseInt(args[1]);
        FoldLineSet set = foldLineSet(args, 2, count);
        set.del_V_all_cc();
        printFoldLineSet(set);
    }

    private static void foldLineFix1(String[] args) {
        if (args.length < 2) {
            usage("foldline-fix1 expects count and segment payload");
        }

        int count = Integer.parseInt(args[1]);
        FoldLineSet set = foldLineSet(args, 2, count);
        boolean result = Fix1.apply(set);
        System.out.println("result|" + result);
        printFoldLineSetWithSelection(set);
    }

    private static void foldLineFix2(String[] args) {
        if (args.length < 2) {
            usage("foldline-fix2 expects count and segment payload");
        }

        int count = Integer.parseInt(args[1]);
        FoldLineSet set = foldLineSet(args, 2, count);
        Fix2.apply(set);
        printFoldLineSetWithSelection(set);
    }

    private static void foldLineSetColor(String[] args) {
        if (args.length < 4) {
            usage("foldline-set-color expects color, comma-separated indices, count, and segment payload");
        }

        LineColor color = LineColor.fromNumber(Integer.parseInt(args[1]));
        int count = Integer.parseInt(args[3]);
        FoldLineSet set = foldLineSet(args, 4, count);
        List<LineSegment> lines = selectedFoldLines(set, args[2]);
        int changed = set.setColor(lines, color);
        System.out.println("changed|" + changed);
        printFoldLineSet(set);
    }

    private static void foldLineChangeType(String[] args) {
        if (args.length < 3) {
            usage("foldline-change-type expects index, count, and segment payload");
        }

        int index = Integer.parseInt(args[1]);
        int count = Integer.parseInt(args[2]);
        FoldLineSet set = foldLineSet(args, 3, count);
        boolean changed = false;
        if (index >= 0 && index < set.getTotal()) {
            LineSegment segment = new LineSegment(set.get(index + 1));
            LineColor color = segment.getColor();
            if (color.isFoldingLine()) {
                set.setColor(segment, color.advanceFolding());
                changed = true;
            }
        }
        System.out.println("changed|" + changed);
        printFoldLineSet(set);
    }

    private static void foldLineMakeColor(String[] args) {
        if (args.length < 4) {
            usage("foldline-make-color expects color, comma-separated indices, count, and segment payload");
        }

        LineColor color = LineColor.fromNumber(Integer.parseInt(args[1]));
        int count = Integer.parseInt(args[3]);
        FoldLineSet set = foldLineSet(args, 4, count);
        List<LineSegment> lines = selectedFoldLines(set, args[2])
                .stream()
                .filter(line -> line.getColor() != color)
                .toList();
        int changed = 0;
        if (!lines.isEmpty()) {
            changed = set.setColor(lines, color);
            Fix2.apply(set);
        }
        System.out.println("changed|" + changed);
        printFoldLineSet(set);
    }

    private static void foldLineMakeAux(String[] args) {
        if (args.length < 3) {
            usage("foldline-make-aux expects comma-separated indices, count, and segment payload");
        }

        int count = Integer.parseInt(args[2]);
        FoldLineSet set = foldLineSet(args, 3, count);
        List<LineSegment> lines = selectedFoldLines(set, args[1])
                .stream()
                .filter(line -> line.getColor().isFoldingLine())
                .toList();

        for (LineSegment line : lines) {
            LineSegment addSen = line.withColor(LineColor.CYAN_3);
            set.deleteLine(line);
            set.addLine(addSen);
        }
        if (!lines.isEmpty()) {
            set.divideLineSegmentWithNewLines(set.getTotal() - lines.size(), set.getTotal());
        }

        System.out.println("changed|" + lines.size());
        printFoldLineSet(set);
    }

    private static void foldLineChangeMv(String[] args) {
        if (args.length < 3) {
            usage("foldline-change-mv expects comma-separated indices, count, and segment payload");
        }

        int count = Integer.parseInt(args[2]);
        FoldLineSet set = foldLineSet(args, 3, count);
        for (LineSegment line : selectedFoldLines(set, args[1])) {
            set.setColor(line, line.getColor().changeMV());
        }
        printFoldLineSet(set);
    }

    private static void foldLineAdvanceType(String[] args) {
        if (args.length < 3) {
            usage("foldline-advance-type expects index, count, and segment payload");
        }

        int index = Integer.parseInt(args[1]);
        int count = Integer.parseInt(args[2]);
        FoldLineSet set = foldLineSet(args, 3, count);
        boolean result = false;
        if (index >= 0 && index < set.getTotal()) {
            LineSegment lineSegment = set.get(index + 1);
            set.deleteLine(lineSegment);

            LineColor color = lineSegment.getColor();
            int selected = lineSegment.getSelected();
            if ((color == LineColor.BLACK_0) && (selected == 0)) {
                lineSegment.setSelected(2);
            } else if ((color == LineColor.BLACK_0) && (selected == 2)) {
                lineSegment = lineSegment.withColor(LineColor.RED_1);
                lineSegment.setSelected(0);
            } else if ((color == LineColor.RED_1) && (selected == 0)) {
                lineSegment = lineSegment.withColor(LineColor.BLUE_2);
            } else if ((color == LineColor.BLUE_2) && (selected == 0)) {
                lineSegment = lineSegment.withColor(LineColor.BLACK_0);
            }

            set.addLine(lineSegment);
            result = true;
        }
        System.out.println("result|" + result);
        printFoldLineSetWithSelection(set);
    }

    private static void foldLineAlternateMv(String[] args) {
        if (args.length < 8) {
            usage("foldline-alternate-mv expects start color, guide segment, count, and segment payload");
        }

        LineColor startColor = LineColor.fromNumber(Integer.parseInt(args[1]));
        LineSegment guide = new LineSegment(
                new Point(parse(args[2]), parse(args[3])),
                new Point(parse(args[4]), parse(args[5])),
                LineColor.fromNumber(Integer.parseInt(args[6])));
        int count = Integer.parseInt(args[7]);
        FoldLineSet set = foldLineSet(args, 8, count);
        int changed = 0;

        if (Epsilon.high.gt0(guide.determineLength())) {
            SortingBox<LineSegment> sorted = new SortingBox<>();
            for (LineSegment line : set.getLineSegmentsIterable()) {
                if (OritaCalc.isLineSegmentOverlapping(line, guide)) {
                    sorted.addByWeight(line, OritaCalc.determineLineSegmentDistance(guide.getA(), line));
                }
            }

            LineColor color = startColor;
            for (int i = 1; i <= sorted.getTotal(); i++) {
                set.setColor(sorted.getValue(i), color);
                changed++;
                if (color == LineColor.RED_1) {
                    color = LineColor.BLUE_2;
                } else if (color == LineColor.BLUE_2) {
                    color = LineColor.RED_1;
                }
            }
        }

        System.out.println("changed|" + changed);
        printFoldLineSet(set);
    }

    private static void foldLineAlternateMvCrossing(String[] args) {
        if (args.length < 8) {
            usage("foldline-alternate-mv-crossing expects start color, guide segment, count, and segment payload");
        }

        LineColor startColor = LineColor.fromNumber(Integer.parseInt(args[1]));
        LineSegment guide = new LineSegment(
                new Point(parse(args[2]), parse(args[3])),
                new Point(parse(args[4]), parse(args[5])),
                LineColor.fromNumber(Integer.parseInt(args[6])));
        int count = Integer.parseInt(args[7]);
        FoldLineSet set = foldLineSet(args, 8, count);
        int changed = 0;

        if (Epsilon.high.gt0(guide.determineLength())) {
            SortingBox<LineSegment> sorted = new SortingBox<>();
            for (LineSegment line : set.getLineSegmentsIterable()) {
                LineSegment.Intersection intersection = OritaCalc.determineLineSegmentIntersection(
                        line,
                        guide,
                        Epsilon.UNKNOWN_1EN4);
                if (!(intersection == LineSegment.Intersection.INTERSECTS_1
                        || intersection == LineSegment.Intersection.INTERSECTS_TSHAPE_S2_VERTICAL_BAR_27
                        || intersection == LineSegment.Intersection.INTERSECTS_TSHAPE_S2_VERTICAL_BAR_28)) {
                    continue;
                }
                sorted.addByWeight(
                        line,
                        OritaCalc.distance(guide.getB(), OritaCalc.findIntersection(line, guide)));
            }

            LineColor color = startColor;
            for (int i = 1; i <= sorted.getTotal(); i++) {
                set.setColor(sorted.getValue(i), color);
                changed++;
                if (color == LineColor.RED_1) {
                    color = LineColor.BLUE_2;
                } else if (color == LineColor.BLUE_2) {
                    color = LineColor.RED_1;
                }
            }
        }

        System.out.println("changed|" + changed);
        printFoldLineSet(set);
    }

    private static void foldLineSelectAll(String[] args) {
        if (args.length < 4) {
            usage("foldline-select-all expects action, preselected indices, count, and segment payload");
        }

        String action = args[1];
        int count = Integer.parseInt(args[3]);
        FoldLineSet set = foldLineSet(args, 4, count);
        applySelectedIndices(set, args[2], 2);

        switch (action) {
            case "select" -> set.select_all();
            case "unselect" -> set.unselect_all();
            default -> usage("unknown select-all action: " + action);
        }

        printFoldLineSetWithSelection(set);
    }

    private static void foldLineSelectIndices(String[] args) {
        if (args.length < 5) {
            usage("foldline-select-indices expects action, indices, preselected indices, count, and segment payload");
        }

        String action = args[1];
        int count = Integer.parseInt(args[4]);
        FoldLineSet set = foldLineSet(args, 5, count);
        applySelectedIndices(set, args[3], 2);

        for (int index : parseIndices(args[2])) {
            switch (action) {
                case "select" -> set.select(index + 1);
                case "unselect" -> set.get(index + 1).setSelected(0);
                default -> usage("unknown select-indices action: " + action);
            }
        }

        printFoldLineSetWithSelection(set);
    }

    private static void foldLineSelectBox(String[] args) {
        if (args.length < 5) {
            usage("foldline-select-box expects action, preselected indices, vertex count, vertices, count, and segment payload");
        }

        String action = args[1];
        int vertexCount = Integer.parseInt(args[3]);
        int countOffset = 4 + vertexCount * 2;
        int count = Integer.parseInt(args[countOffset]);
        FoldLineSet set = foldLineSet(args, countOffset + 1, count);
        applySelectedIndices(set, args[2], 2);
        Polygon polygon = polygon(args, 4, vertexCount);

        for (LineSegment line : set.lineSegmentsInside(polygon)) {
            switch (action) {
                case "select" -> line.setSelected(2);
                case "unselect" -> line.setSelected(0);
                default -> usage("unknown select-box action: " + action);
            }
        }

        printFoldLineSetWithSelection(set);
    }

    private static void foldLineSelectPolygon(String[] args) {
        if (args.length < 5) {
            usage("foldline-select-polygon expects action, preselected indices, vertex count, vertices, count, and segment payload");
        }

        String action = args[1];
        int vertexCount = Integer.parseInt(args[3]);
        int countOffset = 4 + vertexCount * 2;
        int count = Integer.parseInt(args[countOffset]);
        FoldLineSet set = foldLineSet(args, countOffset + 1, count);
        applySelectedIndices(set, args[2], 2);
        Polygon polygon = polygon(args, 4, vertexCount);

        switch (action) {
            case "select" -> set.select_Takakukei(polygon, "select");
            case "unselect" -> set.select_Takakukei(polygon, "unselectAction");
            default -> usage("unknown select-polygon action: " + action);
        }

        printFoldLineSetWithSelection(set);
    }

    private static void foldLineSelectLx(String[] args) {
        if (args.length < 9) {
            usage("foldline-select-lx expects action, preselected indices, selection segment, count, and segment payload");
        }

        String action = args[1];
        LineSegment selection = new LineSegment(
                new Point(parse(args[3]), parse(args[4])),
                new Point(parse(args[5]), parse(args[6])),
                LineColor.fromNumber(Integer.parseInt(args[7])));
        int count = Integer.parseInt(args[8]);
        FoldLineSet set = foldLineSet(args, 9, count);
        applySelectedIndices(set, args[2], 2);

        switch (action) {
            case "select" -> set.select_lX(selection, "select_lX");
            case "unselect" -> set.select_lX(selection, "unselect_lX");
            default -> usage("unknown select-lx action: " + action);
        }

        printFoldLineSetWithSelection(set);
    }

    private static void foldLineSelectConnected(String[] args) {
        if (args.length < 5) {
            usage("foldline-select-connected expects point, preselected indices, count, and segment payload");
        }

        Point point = new Point(parse(args[1]), parse(args[2]));
        int count = Integer.parseInt(args[4]);
        FoldLineSet set = foldLineSet(args, 5, count);
        applySelectedIndices(set, args[3], 2);
        set.selectProbablyConnected(point);
        printFoldLineSetWithSelection(set);
    }

    private static void foldLineDeleteSelected(String[] args) {
        if (args.length < 3) {
            usage("foldline-delete-selected expects preselected indices, count, and segment payload");
        }

        int count = Integer.parseInt(args[2]);
        FoldLineSet set = foldLineSet(args, 3, count);
        applySelectedIndices(set, args[1], 2);
        set.delSelectedLineSegmentFast();
        printFoldLineSetWithSelection(set);
    }

    private static void foldLineReplaceType(String[] args) {
        if (args.length < 5) {
            usage("foldline-replace-type expects from type, to type, indices, count, and segment payload");
        }

        CustomLineTypes from = CustomLineTypes.from(Integer.parseInt(args[1]));
        CustomLineTypes to = CustomLineTypes.from(Integer.parseInt(args[2]));
        int count = Integer.parseInt(args[4]);
        FoldLineSet set = foldLineSet(args, 5, count);
        List<LineSegment> lines = selectedFoldLines(set, args[3])
                .stream()
                .filter(line -> from.matches(line.getColor()))
                .toList();
        int changed = set.setColor(lines, to.getLineColor());
        System.out.println("changed|" + changed);
        printFoldLineSetWithSelection(set);
    }

    private static void foldLineDeleteType(String[] args) {
        if (args.length < 4) {
            usage("foldline-delete-type expects line type, indices, count, and segment payload");
        }

        CustomLineTypes lineType = CustomLineTypes.from(Integer.parseInt(args[1]));
        int count = Integer.parseInt(args[3]);
        FoldLineSet set = foldLineSet(args, 4, count);
        List<LineSegment> lines = selectedFoldLines(set, args[2])
                .stream()
                .filter(line -> lineType.matches(line.getColor()))
                .toList();
        for (LineSegment line : lines) {
            set.deleteLine(line);
        }
        printFoldLineSetWithSelection(set);
    }

    private static void foldLineTranslate(String[] args) {
        if (args.length < 4) {
            usage("foldline-translate expects dx, dy, count, and segment payload");
        }

        double dx = parse(args[1]);
        double dy = parse(args[2]);
        int count = Integer.parseInt(args[3]);
        FoldLineSet set = foldLineSet(args, 4, count);
        set.move(dx, dy);
        printFoldLineSetWithSelection(set);
    }

    private static void foldLineTransformSelected(String[] args) {
        if (args.length < 6) {
            usage("foldline-transform-selected expects mode, dx, dy, preselected indices, count, and segment payload");
        }

        String mode = args[1];
        double dx = parse(args[2]);
        double dy = parse(args[3]);
        int count = Integer.parseInt(args[5]);
        FoldLineSet set = foldLineSet(args, 6, count);
        applySelectedIndices(set, args[4], 2);

        if (Epsilon.high.gt0(new Point(dx, dy).distance(new Point(0.0, 0.0)))) {
            FoldLineSet selected = new FoldLineSet();
            Save save = SaveProvider.createInstance();
            set.getMemoSelectOption(save, 2);
            selected.setSave(save);

            switch (mode) {
                case "move" -> set.delSelectedLineSegmentFast();
                case "copy" -> {}
                default -> usage("unknown transform-selected mode: " + mode);
            }

            selected.move(dx, dy);
            if (mode.equals("copy")) {
                selected.unselect_all();
            }

            int totalOld = set.getTotal();
            Save movedSave = SaveProvider.createInstance();
            selected.getSave(movedSave);
            set.addSave(movedSave);
            int totalNew = set.getTotal();
            set.divideLineSegmentWithNewLines(totalOld, totalNew);
            set.unselect_all();
        }

        printFoldLineSetWithSelection(set);
    }

    private static void foldLineTransformSelected4p(String[] args) {
        if (args.length < 12) {
            usage("foldline-transform-selected-4p expects mode, four points, preselected indices, count, and segment payload");
        }

        String mode = args[1];
        Point originalA = new Point(parse(args[2]), parse(args[3]));
        Point originalB = new Point(parse(args[4]), parse(args[5]));
        Point targetA = new Point(parse(args[6]), parse(args[7]));
        Point targetB = new Point(parse(args[8]), parse(args[9]));
        int count = Integer.parseInt(args[11]);
        FoldLineSet set = foldLineSet(args, 12, count);
        applySelectedIndices(set, args[10], 2);

        FoldLineSet selected = new FoldLineSet();
        Save save = SaveProvider.createInstance();
        set.getMemoSelectOption(save, 2);
        selected.setSave(save);

        switch (mode) {
            case "move" -> set.delSelectedLineSegmentFast();
            case "copy" -> {}
            default -> usage("unknown transform-selected-4p mode: " + mode);
        }

        selected.move(originalA, originalB, targetA, targetB);
        if (mode.equals("copy")) {
            selected.unselect_all();
        }

        int totalOld = set.getTotal();
        Save movedSave = SaveProvider.createInstance();
        selected.getSave(movedSave);
        set.addSave(movedSave);
        int totalNew = set.getTotal();
        set.divideLineSegmentWithNewLines(totalOld, totalNew);
        set.unselect_all();
        printFoldLineSetWithSelection(set);
    }

    private static void foldLineExtendToIntersection(String[] args) {
        if (args.length < 7) {
            usage("foldline-extend-to-intersection expects segment, count, and segment payload");
        }

        LineSegment segment = new LineSegment(
                new Point(parse(args[1]), parse(args[2])),
                new Point(parse(args[3]), parse(args[4])),
                LineColor.fromNumber(Integer.parseInt(args[5])));
        int count = Integer.parseInt(args[6]);
        FoldLineSet set = foldLineSet(args, 7, count);
        LineSegment result = OritaCalc.extendToIntersectionPoint_2(set, segment);
        System.out.println("result|"
                + result.determineAX() + "|"
                + result.determineAY() + "|"
                + result.determineBX() + "|"
                + result.determineBY() + "|"
                + result.getColor().getNumber());
    }

    private static void foldLineDrawCrease(String[] args) {
        if (args.length < 8) {
            usage("foldline-draw-crease expects target, segment, count, and segment payload");
        }

        String target = args[1];
        LineSegment segment = new LineSegment(
                new Point(parse(args[2]), parse(args[3])),
                new Point(parse(args[4]), parse(args[5])),
                LineColor.fromNumber(Integer.parseInt(args[6])));
        int count = Integer.parseInt(args[7]);
        FoldLineSet set = foldLineSet(args, 8, count);
        FoldLineSet aux = new FoldLineSet();
        boolean changed = false;
        if (Epsilon.high.gt0(segment.determineLength())) {
            switch (target) {
                case "fold" -> addLineSegmentLikeWorker(set, segment);
                case "aux" -> aux.addLine(segment);
                default -> usage("unknown draw-crease target: " + target);
            }
            changed = true;
        }

        System.out.println("changed|" + changed);
        printFoldLineSet(set);
        printAuxLineSet(aux);
    }

    private static void foldLineDrawSymmetric(String[] args) {
        if (args.length < 8) {
            usage("foldline-draw-symmetric expects axis segment, preselected indices, count, and segment payload");
        }

        LineSegment axis = new LineSegment(
                new Point(parse(args[1]), parse(args[2])),
                new Point(parse(args[3]), parse(args[4])),
                LineColor.fromNumber(Integer.parseInt(args[5])));
        int count = Integer.parseInt(args[7]);
        FoldLineSet set = foldLineSet(args, 8, count);
        applySelectedIndices(set, args[6], 2);

        int oldTotal = set.getTotal();
        int mirrored = 0;
        for (LineSegment line : set.getLineSegmentsCollection()) {
            if (line.getSelected() == 2) {
                LineSegment add = OritaCalc
                        .findLineSymmetryLineSegment(line, axis)
                        .withColor(line.getColor());
                set.addLine(add);
                mirrored++;
            }
        }
        int newTotal = set.getTotal();
        set.divideLineSegmentWithNewLines(oldTotal, newTotal);
        set.unselect_all();

        System.out.println("mirrored|" + mirrored);
        printFoldLineSetWithSelection(set);
    }

    private static void foldLineDrawPoint(String[] args) {
        if (args.length < 6) {
            usage("foldline-draw-point expects index, target point, selection distance, count, and segment payload");
        }

        int index = Integer.parseInt(args[1]);
        Point target = new Point(parse(args[2]), parse(args[3]));
        double selectionDistance = parse(args[4]);
        int count = Integer.parseInt(args[5]);
        FoldLineSet set = foldLineSet(args, 6, count);
        boolean changed = false;
        if (index >= 0 && index < set.getTotal()) {
            LineSegment segment = new LineSegment(set.get(index + 1));
            if (OritaCalc.determineLineSegmentDistance(target, segment) <= selectionDistance) {
                Point projection = OritaCalc.findProjection(OritaCalc.lineSegmentToStraightLine(segment), target);
                if (OritaCalc.isInside(segment.getA(), projection, segment.getB()) == 2) {
                    set.applyLineSegmentDivide(segment, projection);
                    changed = true;
                }
            }
        }

        System.out.println("changed|" + changed);
        printFoldLineSet(set);
    }

    private static void foldLineCircleDraw(String[] args) {
        if (args.length != 5) {
            usage("foldline-circle-draw expects center and radius point");
        }

        Point center = new Point(parse(args[1]), parse(args[2]));
        Point radiusPoint = new Point(parse(args[3]), parse(args[4]));
        FoldLineSet set = new FoldLineSet();
        set.addCircle(center.getX(), center.getY(), OritaCalc.distance(center, radiusPoint), LineColor.CYAN_3);
        System.out.println("changed|true");
        printCircleSet(set);
    }

    private static void foldLineCircleDrawFree(String[] args) {
        if (args.length != 5) {
            usage("foldline-circle-draw-free expects center and radius point");
        }

        Point center = new Point(parse(args[1]), parse(args[2]));
        Point radiusPoint = new Point(parse(args[3]), parse(args[4]));
        FoldLineSet set = new FoldLineSet();
        boolean changed = false;
        if (!center.equals(radiusPoint)) {
            set.addCircle(center.getX(), center.getY(), OritaCalc.distance(center, radiusPoint), LineColor.CYAN_3);
            changed = true;
        }
        System.out.println("changed|" + changed);
        printCircleSet(set);
    }

    private static void foldLineCircleThreePoint(String[] args) {
        if (args.length != 7) {
            usage("foldline-circle-three-point expects three points");
        }

        Point p1 = new Point(parse(args[1]), parse(args[2]));
        Point p2 = new Point(parse(args[3]), parse(args[4]));
        Point p3 = new Point(parse(args[5]), parse(args[6]));
        FoldLineSet set = new FoldLineSet();
        boolean changed = false;

        LineSegment sen1 = new LineSegment(p1, p2);
        LineSegment sen2 = new LineSegment(p2, p3);
        LineSegment sen3 = new LineSegment(p3, p1);

        if (!isFlatAngle(OritaCalc.angle(sen1, sen2))
                && !isFlatAngle(OritaCalc.angle(sen2, sen3))
                && !isFlatAngle(OritaCalc.angle(sen3, sen1))) {
            StraightLine t1 = new StraightLine(sen1)
                    .orthogonalize(OritaCalc.internalDivisionRatio(sen1.getA(), sen1.getB(), 1.0, 1.0));
            StraightLine t2 = new StraightLine(sen2)
                    .orthogonalize(OritaCalc.internalDivisionRatio(sen2.getA(), sen2.getB(), 1.0, 1.0));
            Point center = OritaCalc.findIntersection(t1, t2);
            set.addCircle(center.getX(), center.getY(), OritaCalc.distance(p1, center), LineColor.CYAN_3);
            changed = true;
        }

        System.out.println("changed|" + changed);
        printCircleSet(set);
    }

    private static boolean isFlatAngle(double angle) {
        return Math.abs(angle) < Epsilon.UNKNOWN_1EN6
                || Math.abs(angle - 180.0) < Epsilon.UNKNOWN_1EN6
                || Math.abs(angle - 360.0) < Epsilon.UNKNOWN_1EN6;
    }

    private static void foldLineCircleSeparate(String[] args) {
        if (args.length != 7) {
            usage("foldline-circle-separate expects center and radius segment endpoints");
        }

        Point center = new Point(parse(args[1]), parse(args[2]));
        Point a = new Point(parse(args[3]), parse(args[4]));
        Point b = new Point(parse(args[5]), parse(args[6]));
        FoldLineSet set = new FoldLineSet();
        set.addCircle(center.getX(), center.getY(), OritaCalc.distance(a, b), LineColor.CYAN_3);
        System.out.println("changed|true");
        printCircleSet(set);
    }

    private static void foldLineCircleConcentric(String[] args) {
        if (args.length != 9) {
            usage("foldline-circle-concentric expects circle and radius segment endpoints");
        }

        Circle original = circle(args, 1);
        Point a = new Point(parse(args[5]), parse(args[6]));
        Point b = new Point(parse(args[7]), parse(args[8]));
        FoldLineSet set = new FoldLineSet();
        set.addCircle(
                original.getX(),
                original.getY(),
                original.getR() + OritaCalc.distance(a, b),
                LineColor.CYAN_3);
        System.out.println("changed|true");
        printCircleSet(set);
    }

    private static void foldLineCircleConcentricSelect(String[] args) {
        if (args.length != 14) {
            usage("foldline-circle-concentric-select expects candidate index, target circle, and two reference circles");
        }

        int candidateIndex = Integer.parseInt(args[1]);
        Circle target = circle(args, 2);
        Circle reference1 = circle(args, 6);
        Circle reference2 = circle(args, 10);
        List<Circle> candidates = concentricSelectCandidates(target, reference1, reference2);
        FoldLineSet set = new FoldLineSet();
        boolean changed = false;
        if (candidateIndex >= 0 && candidateIndex < candidates.size()) {
            Circle selected = new Circle(candidates.get(candidateIndex));
            selected.setColor(LineColor.CYAN_3);
            set.getCircles().add(selected);
            changed = true;
        }
        System.out.println("changed|" + changed);
        printCircleSet(set);
    }

    private static List<Circle> concentricSelectCandidates(Circle target, Circle reference1, Circle reference2) {
        List<Circle> candidates = new ArrayList<>();
        double deltaR = Math.abs(reference2.getR() - reference1.getR());
        if (Epsilon.high.eq0(deltaR)) {
            return candidates;
        }

        double outerR = target.getR() + deltaR;
        double innerR = target.getR() - deltaR;
        candidates.add(new Circle(target.determineCenter(), outerR, LineColor.MAGENTA_5));
        if (Epsilon.high.gt0(innerR)) {
            candidates.add(new Circle(target.determineCenter(), innerR, LineColor.MAGENTA_5));
        }
        return candidates;
    }

    private static void foldLineCircleConcentricTwo(String[] args) {
        if (args.length != 9) {
            usage("foldline-circle-concentric-two expects two circles");
        }

        Circle circle1 = circle(args, 1);
        Circle circle2 = circle(args, 5);
        FoldLineSet set = new FoldLineSet();
        int added = 0;
        if (OritaCalc.circle_to_circle_intersection(circle1, circle2) != Circle.Intersection.TANGENT) {
            double centerLineLength = OritaCalc.distance(circle1.determineCenter(), circle2.determineCenter());
            double concentricOffset = (centerLineLength - circle1.getR() - circle2.getR()) / 2.0;
            set.addCircle(circle1.getX(), circle1.getY(), circle1.getR() + concentricOffset, LineColor.CYAN_3);
            set.addCircle(circle2.getX(), circle2.getY(), circle2.getR() + concentricOffset, LineColor.CYAN_3);
            added = 2;
        }
        System.out.println("added|" + added);
        printCircleSet(set);
    }

    private static void foldLineCircleInvertCircle(String[] args) {
        if (args.length != 9) {
            usage("foldline-circle-invert-circle expects subject circle and inversion circle");
        }

        Circle subject = circle(args, 1);
        Circle inversion = circle(args, 5);
        FoldLineSet set = new FoldLineSet();
        String outcome = invertCircle(set, subject, inversion);
        System.out.println("outcome|" + outcome);
        printFoldLineSet(set);
        printCircleSet(set);
    }

    private static String invertCircle(FoldLineSet set, Circle subject, Circle inversion) {
        if (Math.abs(OritaCalc.distance(subject.determineCenter(), inversion.determineCenter())
                - subject.getR()) < Epsilon.UNKNOWN_1EN7) {
            set.addLine(inversion.turnAround_CircleToLineSegment(subject));
            return "line";
        }

        Circle added = new Circle();
        added.set(inversion.turnAround(subject));
        added.setColor(LineColor.CYAN_3);
        set.getCircles().add(added);
        return "circle";
    }

    private static void foldLineCircleInvertLine(String[] args) {
        if (args.length != 10) {
            usage("foldline-circle-invert-line expects line segment and inversion circle");
        }

        LineSegment subject = segment(args, 1);
        Circle inversion = circle(args, 6);
        FoldLineSet set = new FoldLineSet();
        String outcome = invertLineSegment(set, subject, inversion);
        System.out.println("outcome|" + outcome);
        printFoldLineSet(set);
        printCircleSet(set);
    }

    private static String invertLineSegment(FoldLineSet set, LineSegment subject, Circle inversion) {
        StraightLine ty = new StraightLine(subject);
        if (ty.calculateDistance(inversion.determineCenter()) < Epsilon.UNKNOWN_1EN7) {
            return "none";
        }

        Circle added = new Circle();
        added.set(inversion.turnAround_LineSegmentToCircle(subject));
        added.setColor(LineColor.CYAN_3);
        set.getCircles().add(added);
        return "circle";
    }

    private static void foldLineCircleOrganize(String[] args) {
        if (args.length < 3) {
            usage("foldline-circle-organize expects circles followed by fold lines");
        }

        int circleCount = Integer.parseInt(args[1]);
        int lineCountOffset = 2 + circleCount * 4;
        if (args.length < lineCountOffset + 1) {
            usage("foldline-circle-organize missing line count");
        }
        int lineCount = Integer.parseInt(args[lineCountOffset]);
        int expectedLength = lineCountOffset + 1 + lineCount * 5;
        if (args.length != expectedLength) {
            usage("foldline-circle-organize payload length mismatch");
        }

        FoldLineSet set = new FoldLineSet();
        for (int i = 0; i < circleCount; i++) {
            set.getCircles().add(circle(args, 2 + i * 4));
        }
        for (int i = 0; i < lineCount; i++) {
            LineSegment line = segment(args, lineCountOffset + 1 + i * 5);
            set.addLine(line);
        }

        int before = set.getCircles().size();
        OrganizeCircles.apply(set);
        System.out.println("deleted|" + (before - set.getCircles().size()));
        printCircleSet(set);
    }

    private static void foldLineCircleChangeColor(String[] args) {
        if (args.length < 8) {
            usage("foldline-circle-change-color expects index sets, rgb color, circles, and fold lines");
        }

        List<Integer> circleIndices = parseIndices(args[1]);
        List<Integer> auxLineIndices = parseIndices(args[2]);
        Color color = new Color(
                Integer.parseInt(args[3]),
                Integer.parseInt(args[4]),
                Integer.parseInt(args[5]));
        int circleCount = Integer.parseInt(args[6]);
        int lineCountOffset = 7 + circleCount * 4;
        if (args.length < lineCountOffset + 1) {
            usage("foldline-circle-change-color missing line count");
        }
        int lineCount = Integer.parseInt(args[lineCountOffset]);
        int expectedLength = lineCountOffset + 1 + lineCount * 5;
        if (args.length != expectedLength) {
            usage("foldline-circle-change-color payload length mismatch");
        }

        FoldLineSet set = new FoldLineSet();
        for (int i = 0; i < circleCount; i++) {
            set.getCircles().add(circle(args, 7 + i * 4));
        }
        for (int i = 0; i < lineCount; i++) {
            set.addLine(segment(args, lineCountOffset + 1 + i * 5));
        }

        int changed = 0;
        for (int index : circleIndices) {
            if (index >= 0 && index < set.getCircles().size()) {
                set.setCircleCustomizedColor(set.getCircles().get(index), color);
                changed++;
            }
        }
        for (int index : auxLineIndices) {
            if (index >= 0 && index < set.getTotal()) {
                LineSegment line = set.get(index + 1);
                if (line.getColor() == LineColor.CYAN_3) {
                    set.setCustomized(line, color);
                    changed++;
                }
            }
        }

        System.out.println("changed|" + changed);
        printCircleSet(set);
        printFoldLineSetWithCustomization(set);
    }

    private static void foldLineCircleTangentPoint(String[] args) {
        if (args.length < 8) {
            usage("foldline-circle-tangent-point expects point, circle, line count, and fold lines");
        }

        Point point = new Point(parse(args[1]), parse(args[2]));
        Circle circle = circle(args, 3);
        int lineCount = Integer.parseInt(args[7]);
        FoldLineSet set = foldLineSet(args, 8, lineCount);
        printLineSegmentsList(tangentPointCircle(set, point, circle));
    }

    private static List<LineSegment> tangentPointCircle(FoldLineSet set, Point point, Circle circle) {
        List<LineSegment> indicators = new ArrayList<>();
        if (Math.abs(circle.getR() - OritaCalc.distance(circle.determineCenter(), point)) < Epsilon.UNKNOWN_1EN7) {
            LineSegment projectionLine = new LineSegment(circle.determineCenter(), point);
            indicators.add(OritaCalc.fullExtendUntilHit(set, new LineSegment(point,
                    OritaCalc.findProjection(OritaCalc.moveParallel(projectionLine, 1), point),
                    LineColor.PURPLE_8)));
            indicators.add(OritaCalc.fullExtendUntilHit(set, new LineSegment(point,
                    OritaCalc.findProjection(OritaCalc.moveParallel(projectionLine, -1), point),
                    LineColor.PURPLE_8)));
            return indicators;
        }
        LineSegment diameter = new LineSegment(point, circle.determineCenter());
        Circle constructCir = new Circle(diameter, LineColor.GREEN_6);
        LineSegment connectSegment = OritaCalc
                .circle_to_circle_no_intersection_wo_musubu_lineSegment(constructCir, circle);
        indicators.add(new LineSegment(point, connectSegment.getA(), LineColor.PURPLE_8));
        indicators.add(new LineSegment(point, connectSegment.getB(), LineColor.PURPLE_8));
        return indicators;
    }

    private static void foldLineCircleTangentTwo(String[] args) {
        if (args.length != 9) {
            usage("foldline-circle-tangent-two expects two circles");
        }

        printLineSegmentsList(tangentTwoCircles(circle(args, 1), circle(args, 5)));
    }

    private static List<LineSegment> tangentTwoCircles(Circle circle1, Circle circle2) {
        List<LineSegment> indicators = new ArrayList<>();
        Point c1 = circle1.determineCenter();
        Point c2 = circle2.determineCenter();

        double x1 = circle1.getX();
        double y1 = circle1.getY();
        double r1 = circle1.getR();
        double x2 = circle2.getX();
        double y2 = circle2.getY();
        double r2 = circle2.getR();
        double xp = x2 - x1;
        double yp = y2 - y1;
        double distanceSquared = xp * xp + yp * yp;
        double radiusDifferenceSquared = (r1 - r2) * (r1 - r2);
        double radiusSumSquared = (r1 + r2) * (r1 + r2);

        if (c1.distance(c2) < Epsilon.UNKNOWN_1EN6) return indicators;
        if (distanceSquared < radiusDifferenceSquared) return indicators;

        if (Math.abs(distanceSquared - radiusDifferenceSquared) < Epsilon.UNKNOWN_1EN7) {
            Point kouten = OritaCalc.internalDivisionRatio(c1, c2, -r1, r2);
            StraightLine ty = new StraightLine(c1, kouten).orthogonalize(kouten);
            indicators.add(OritaCalc.circle_to_straightLine_no_intersect_wo_connect_LineSegment(
                    new Circle(kouten, (r1 + r2) / 2.0, LineColor.BLACK_0), ty)
                    .withColor(LineColor.PURPLE_8));
            return indicators;
        }

        indicators.addAll(externalTangentSegments(x1, y1, x2, y2, r1, r2, xp, yp, distanceSquared));
        if (radiusDifferenceSquared < distanceSquared && distanceSquared < radiusSumSquared) {
            return indicators;
        }
        if (Math.abs(distanceSquared - radiusSumSquared) < Epsilon.UNKNOWN_1EN7) {
            Point kouten = OritaCalc.internalDivisionRatio(c1, c2, r1, r2);
            StraightLine ty = new StraightLine(c1, kouten).orthogonalize(kouten);
            indicators.add(OritaCalc.circle_to_straightLine_no_intersect_wo_connect_LineSegment(
                    new Circle(kouten, (r1 + r2) / 2.0, LineColor.BLACK_0), ty)
                    .withColor(LineColor.PURPLE_8));
            return indicators;
        }
        if (radiusSumSquared < distanceSquared) {
            indicators.addAll(internalTangentSegments(x1, y1, x2, y2, r1, r2, xp, yp, distanceSquared));
        }
        return indicators;
    }

    private static List<LineSegment> externalTangentSegments(double x1, double y1, double x2, double y2,
                                                             double r1, double r2, double xp, double yp,
                                                             double distanceSquared) {
        double root = Math.sqrt(distanceSquared - (r1 - r2) * (r1 - r2));
        double xq1 = r1 * (xp * (r1 - r2) + yp * root) / distanceSquared;
        double yq1 = r1 * (yp * (r1 - r2) - xp * root) / distanceSquared;
        double xq2 = r1 * (xp * (r1 - r2) - yp * root) / distanceSquared;
        double yq2 = r1 * (yp * (r1 - r2) + xp * root) / distanceSquared;
        return tangentSegmentsFromOffsets(x1, y1, x2, y2, new double[][]{{xq1, yq1}, {xq2, yq2}});
    }

    private static List<LineSegment> internalTangentSegments(double x1, double y1, double x2, double y2,
                                                             double r1, double r2, double xp, double yp,
                                                             double distanceSquared) {
        double root = Math.sqrt(distanceSquared - (r1 + r2) * (r1 + r2));
        double xq3 = r1 * (xp * (r1 + r2) + yp * root) / distanceSquared;
        double yq3 = r1 * (yp * (r1 + r2) - xp * root) / distanceSquared;
        double xq4 = r1 * (xp * (r1 + r2) - yp * root) / distanceSquared;
        double yq4 = r1 * (yp * (r1 + r2) + xp * root) / distanceSquared;
        return tangentSegmentsFromOffsets(x1, y1, x2, y2, new double[][]{{xq3, yq3}, {xq4, yq4}});
    }

    private static List<LineSegment> tangentSegmentsFromOffsets(double x1, double y1, double x2, double y2, double[][] offsets) {
        List<LineSegment> indicators = new ArrayList<>();
        for (double[] offset : offsets) {
            double xr = offset[0] + x1;
            double yr = offset[1] + y1;
            StraightLine t = new StraightLine(x1, y1, xr, yr).orthogonalize(new Point(xr, yr));
            indicators.add(new LineSegment(new Point(xr, yr),
                    OritaCalc.findProjection(t, new Point(x2, y2)),
                    LineColor.PURPLE_8));
        }
        return indicators;
    }

    private static void foldLineDivideCount(String[] args) {
        if (args.length < 8) {
            usage("foldline-divide-count expects division count, segment, count, and segment payload");
        }

        int divisionCount = Integer.parseInt(args[1]);
        LineSegment segment = new LineSegment(
                new Point(parse(args[2]), parse(args[3])),
                new Point(parse(args[4]), parse(args[5])),
                LineColor.fromNumber(Integer.parseInt(args[6])));
        int count = Integer.parseInt(args[7]);
        FoldLineSet set = foldLineSet(args, 8, count);

        if (divisionCount != 0 && Epsilon.high.gt0(segment.determineLength())) {
            for (int i = 0; i <= divisionCount - 1; i++) {
                double ax = ((double) (divisionCount - i) * segment.determineAX()
                        + (double) i * segment.determineBX()) / ((double) divisionCount);
                double ay = ((double) (divisionCount - i) * segment.determineAY()
                        + (double) i * segment.determineBY()) / ((double) divisionCount);
                double bx = ((double) (divisionCount - i - 1) * segment.determineAX()
                        + (double) (i + 1) * segment.determineBX()) / ((double) divisionCount);
                double by = ((double) (divisionCount - i - 1) * segment.determineAY()
                        + (double) (i + 1) * segment.determineBY()) / ((double) divisionCount);
                addLineSegmentLikeWorker(set, new LineSegment(ax, ay, bx, by).withColor(segment.getColor()));
            }
        }

        printFoldLineSet(set);
    }

    private static void foldLineDivideRatio(String[] args) {
        if (args.length < 9) {
            usage("foldline-divide-ratio expects ratio s, ratio t, segment, count, and segment payload");
        }

        double ratioS = parse(args[1]);
        double ratioT = parse(args[2]);
        LineSegment segment = new LineSegment(
                new Point(parse(args[3]), parse(args[4])),
                new Point(parse(args[5]), parse(args[6])),
                LineColor.fromNumber(Integer.parseInt(args[7])));
        int count = Integer.parseInt(args[8]);
        FoldLineSet set = foldLineSet(args, 9, count);

        if (Epsilon.high.gt0(segment.determineLength())) {
            LineSegment dragSegment = segment.withAB(segment.getB(), segment.getA());
            if ((ratioS == 0.0 && ratioT != 0.0) || (ratioS != 0.0 && ratioT == 0.0)) {
                addLineSegmentLikeWorker(set, dragSegment);
            }
            if (ratioS != 0.0 && ratioT != 0.0) {
                double nx = (ratioT * dragSegment.determineBX()
                        + ratioS * dragSegment.determineAX())
                        / (ratioS + ratioT);
                double ny = (ratioT * dragSegment.determineBY()
                        + ratioS * dragSegment.determineAY())
                        / (ratioS + ratioT);
                LineSegment sAd = new LineSegment().withColor(segment.getColor());
                addLineSegmentLikeWorker(set, sAd.withCoordinates(
                        dragSegment.determineAX(),
                        dragSegment.determineAY(),
                        nx,
                        ny));
                addLineSegmentLikeWorker(set, sAd.withCoordinates(
                        dragSegment.determineBX(),
                        dragSegment.determineBY(),
                        nx,
                        ny));
            }
        }

        printFoldLineSet(set);
    }

    private static void measureLength(String[] args) {
        if (args.length != 5) {
            usage("measure-length expects two points");
        }

        Point a = new Point(parse(args[1]), parse(args[2]));
        Point b = new Point(parse(args[3]), parse(args[4]));
        System.out.println("length|" + a.distance(b));
    }

    private static void measureAngle(String[] args) {
        if (args.length != 7) {
            usage("measure-angle expects three points");
        }

        Point a = new Point(parse(args[1]), parse(args[2]));
        Point center = new Point(parse(args[3]), parse(args[4]));
        Point b = new Point(parse(args[5]), parse(args[6]));
        System.out.println("angle|" + OritaCalc.angle(center, a, center, b));
    }

    private static void orhImportSummary(String[] args) throws Exception {
        if (args.length != 2) {
            usage("orh-import-summary expects a file path");
        }

        Save save = new OrhImporter().doImport(new File(args[1]));
        printSaveSummary(save);
    }

    private static void objImportSummary(String[] args) throws Exception {
        if (args.length != 2) {
            usage("obj-import-summary expects a file path");
        }

        Save save = new ObjImporter().doImport(new File(args[1]));
        printSaveSummary(save);
    }

    private static void orhExportFixture(String[] args) throws Exception {
        if (args.length != 1) {
            usage("orh-export-fixture does not take arguments");
        }

        File file = File.createTempFile("oriedita-oracle", ".orh");
        try {
            new OrhExporter().doExport(fixtureSave(), file);
            System.out.print(Files.readString(file.toPath()));
        } finally {
            file.delete();
        }
    }

    private static void dxfExportFixture(String[] args) throws Exception {
        if (args.length != 1) {
            usage("dxf-export-fixture does not take arguments");
        }

        File file = File.createTempFile("oriedita-oracle", ".dxf");
        try {
            new DxfExporter().doExport(fixtureSave(), file);
            System.out.print(Files.readString(file.toPath()));
        } finally {
            file.delete();
        }
    }

    private static Save fixtureSave() {
        Save save = SaveProvider.createInstance();
        save.setTitle("oracle");
        save.addLineSegment(new LineSegment(
                new Point(0.0, 0.0),
                new Point(10.0, 0.0),
                LineColor.BLUE_2).withCustomizedColor(new Color(1, 2, 3)));
        save.addCircle(new Circle(5.0, 5.0, 2.0, LineColor.MAGENTA_5));
        save.addAuxLineSegment(new LineSegment(
                new Point(1.0, 1.0),
                new Point(2.0, 2.0),
                LineColor.ORANGE_4));
        GridModel gridModel = new GridModel();
        gridModel.setBaseState(GridModel.State.HIDDEN);
        gridModel.setGridSize(24);
        save.setGridModel(gridModel);
        return save;
    }

    private static LineSegmentSet lineSegmentSet(String[] args, int offset, int count) {
        int expectedLength = offset + count * 5;
        if (args.length != expectedLength) {
            usage("line segment payload expects " + count + " entries of ax ay bx by color");
        }

        LineSegmentSet set = new LineSegmentSet();
        for (int index = 0; index < count; index++) {
            int base = offset + index * 5;
            set.addLine(
                    new Point(parse(args[base]), parse(args[base + 1])),
                    new Point(parse(args[base + 2]), parse(args[base + 3])),
                    LineColor.fromNumber(Integer.parseInt(args[base + 4])));
        }
        return set;
    }

    private static FoldLineSet foldLineSet(String[] args, int offset, int count) {
        int expectedLength = offset + count * 5;
        if (args.length != expectedLength) {
            usage("fold line payload expects " + count + " entries of ax ay bx by color");
        }

        FoldLineSet set = new FoldLineSet();
        for (int index = 0; index < count; index++) {
            int base = offset + index * 5;
            set.addLine(
                    new Point(parse(args[base]), parse(args[base + 1])),
                    new Point(parse(args[base + 2]), parse(args[base + 3])),
                    LineColor.fromNumber(Integer.parseInt(args[base + 4])));
        }
        return set;
    }

    private static LineSegment segment(String[] args, int offset) {
        return new LineSegment(
                new Point(parse(args[offset]), parse(args[offset + 1])),
                new Point(parse(args[offset + 2]), parse(args[offset + 3])),
                LineColor.fromNumber(Integer.parseInt(args[offset + 4])));
    }

    private static Circle circle(String[] args, int offset) {
        return new Circle(
                parse(args[offset]),
                parse(args[offset + 1]),
                parse(args[offset + 2]),
                LineColor.fromNumber(Integer.parseInt(args[offset + 3])));
    }

    private static void addLineSegmentLikeWorker(FoldLineSet set, LineSegment segment) {
        set.addLine(segment);
        int totalOld = set.getTotal();
        set.divideLineSegmentWithNewLines(totalOld - 1, totalOld);
    }

    private static Polygon polygon(String[] args, int offset, int count) {
        List<Point> points = new ArrayList<>();
        for (int index = 0; index < count; index++) {
            int base = offset + index * 2;
            points.add(new Point(parse(args[base]), parse(args[base + 1])));
        }
        return new Polygon(points);
    }

    private static void applySelectedIndices(FoldLineSet set, String indices, int selected) {
        for (int index : parseIndices(indices)) {
            set.get(index + 1).setSelected(selected);
        }
    }

    private static List<Integer> parseIndices(String indices) {
        List<Integer> result = new ArrayList<>();
        if (indices.equals("-") || indices.isEmpty()) {
            return result;
        }

        for (String value : indices.split(",")) {
            if (!value.isEmpty()) {
                result.add(Integer.parseInt(value));
            }
        }
        return result;
    }

    private static void printLineSegmentSet(LineSegmentSet set) {
        System.out.println("lines|" + set.getNumLineSegments());
        for (int index = 0; index < set.getNumLineSegments(); index++) {
            LineSegment segment = set.get(index);
            System.out.println("line|"
                    + segment.determineAX() + "|"
                    + segment.determineAY() + "|"
                    + segment.determineBX() + "|"
                    + segment.determineBY() + "|"
                    + segment.getColor().getNumber());
        }
    }

    private static void printLineSegmentsList(List<LineSegment> segments) {
        System.out.println("lines|" + segments.size());
        for (LineSegment segment : segments) {
            System.out.println("line|"
                    + segment.determineAX() + "|"
                    + segment.determineAY() + "|"
                    + segment.determineBX() + "|"
                    + segment.determineBY() + "|"
                    + segment.getColor().getNumber());
        }
    }

    private static void printFoldLineSet(FoldLineSet set) {
        System.out.println("lines|" + set.getTotal());
        for (int index = 1; index <= set.getTotal(); index++) {
            LineSegment segment = set.get(index);
            System.out.println("line|"
                    + segment.determineAX() + "|"
                    + segment.determineAY() + "|"
                    + segment.determineBX() + "|"
                    + segment.determineBY() + "|"
                    + segment.getColor().getNumber());
        }
    }

    private static void printAuxLineSet(FoldLineSet set) {
        System.out.println("aux|" + set.getTotal());
        for (int index = 1; index <= set.getTotal(); index++) {
            LineSegment segment = set.get(index);
            System.out.println("auxline|"
                    + segment.determineAX() + "|"
                    + segment.determineAY() + "|"
                    + segment.determineBX() + "|"
                    + segment.determineBY() + "|"
                    + segment.getColor().getNumber());
        }
    }

    private static void printCircleSet(FoldLineSet set) {
        System.out.println("circles|" + set.getCircles().size());
        for (Circle circle : set.getCircles()) {
            System.out.println("circle|"
                    + circle.getX() + "|"
                    + circle.getY() + "|"
                    + circle.getR() + "|"
                    + circle.getColor().getNumber() + "|"
                    + circle.getCustomized() + "|"
                    + circle.getCustomizedColor().getRed() + "|"
                    + circle.getCustomizedColor().getGreen() + "|"
                    + circle.getCustomizedColor().getBlue());
        }
    }

    private static void printFoldLineSetWithCustomization(FoldLineSet set) {
        System.out.println("lines|" + set.getTotal());
        for (int index = 1; index <= set.getTotal(); index++) {
            LineSegment segment = set.get(index);
            System.out.println("line|"
                    + segment.determineAX() + "|"
                    + segment.determineAY() + "|"
                    + segment.determineBX() + "|"
                    + segment.determineBY() + "|"
                    + segment.getColor().getNumber() + "|"
                    + segment.getCustomized() + "|"
                    + segment.getCustomizedColor().getRed() + "|"
                    + segment.getCustomizedColor().getGreen() + "|"
                    + segment.getCustomizedColor().getBlue());
        }
    }

    private static void printFoldLineSetWithSelection(FoldLineSet set) {
        System.out.println("lines|" + set.getTotal());
        for (int index = 1; index <= set.getTotal(); index++) {
            LineSegment segment = set.get(index);
            System.out.println("line|"
                    + segment.determineAX() + "|"
                    + segment.determineAY() + "|"
                    + segment.determineBX() + "|"
                    + segment.determineBY() + "|"
                    + segment.getColor().getNumber() + "|"
                    + segment.getSelected());
        }
    }

    private static void printFoldLineSetDeleteSet(Set<Integer> toDelete) {
        System.out.print("delete");
        toDelete.stream().sorted().forEach(index -> System.out.print("|" + (index - 1)));
        System.out.println();
    }

    private static List<LineSegment> selectedFoldLines(FoldLineSet set, String indices) {
        List<LineSegment> lines = new ArrayList<>();
        if (indices.isEmpty()) {
            return lines;
        }
        for (String index : indices.split(",")) {
            lines.add(set.get(Integer.parseInt(index) + 1));
        }
        return lines;
    }

    private static void printSaveSummary(Save save) {
        System.out.println("title|" + nullToEmpty(save.getTitle()));
        System.out.println("lines|" + save.getLineSegments().size());
        for (LineSegment segment : save.getLineSegments()) {
            System.out.println("line|"
                    + segment.determineAX() + "|"
                    + segment.determineAY() + "|"
                    + segment.determineBX() + "|"
                    + segment.determineBY() + "|"
                    + segment.getColor().getNumber() + "|"
                    + segment.getActive().name() + "|"
                    + segment.getSelected() + "|"
                    + segment.getCustomized() + "|"
                    + segment.getCustomizedColor().getRed() + "|"
                    + segment.getCustomizedColor().getGreen() + "|"
                    + segment.getCustomizedColor().getBlue());
        }
        System.out.println("circles|" + save.getCircles().size());
        for (Circle circle : save.getCircles()) {
            System.out.println("circle|"
                    + circle.getX() + "|"
                    + circle.getY() + "|"
                    + circle.getR() + "|"
                    + circle.getColor().getNumber() + "|"
                    + circle.getCustomized() + "|"
                    + circle.getCustomizedColor().getRed() + "|"
                    + circle.getCustomizedColor().getGreen() + "|"
                    + circle.getCustomizedColor().getBlue());
        }
        System.out.println("aux|" + save.getAuxLineSegments().size());
        for (LineSegment segment : save.getAuxLineSegments()) {
            System.out.println("auxline|"
                    + segment.determineAX() + "|"
                    + segment.determineAY() + "|"
                    + segment.determineBX() + "|"
                    + segment.determineBY() + "|"
                    + segment.getColor().getNumber() + "|"
                    + segment.getActive().name() + "|"
                    + segment.getSelected() + "|"
                    + segment.getCustomized() + "|"
                    + segment.getCustomizedColor().getRed() + "|"
                    + segment.getCustomizedColor().getGreen() + "|"
                    + segment.getCustomizedColor().getBlue());
        }
        GridModel grid = save.getGridModel();
        if (grid == null) {
            System.out.println("grid|null");
        } else {
            System.out.println("grid|"
                    + grid.getIntervalGridSize() + "|"
                    + grid.getGridSize() + "|"
                    + grid.getGridXA() + "|"
                    + grid.getGridXB() + "|"
                    + grid.getGridXC() + "|"
                    + grid.getGridYA() + "|"
                    + grid.getGridYB() + "|"
                    + grid.getGridYC() + "|"
                    + grid.getGridAngle() + "|"
                    + grid.getBaseState().getState() + "|"
                    + grid.getVerticalScalePosition() + "|"
                    + grid.getHorizontalScalePosition() + "|"
                    + grid.getDrawDiagonalGridlines());
        }
    }

    private static String nullToEmpty(String value) {
        return value == null ? "" : value;
    }

    private static double parse(String value) {
        return Double.parseDouble(value);
    }

    private static void usage(String message) {
        System.err.println(message);
        System.err.println("usage: OrieditaGeometryOracle intersection <strict|sweet> <default|precision> ax ay bx by cx cy dx dy");
        System.err.println("   or: OrieditaGeometryOracle intersect-divide <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle intersect-divide-pair <i> <j> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-divide-new-lines <originalEnd> <addedEnd> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-divide-fast <i> <j> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-delete-inside <l|lX> <selection ax ay bx by color> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-delete-line-vertex <index> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-delete-lines <indices> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-branch-trim <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-del-v <px> <py> <snapRadius> <vertexRadius> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-del-v-cc <px> <py> <snapRadius> <vertexRadius> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-del-v-pair <i> <j> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-del-v-all <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-del-v-all-cc <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-fix1 <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-fix2 <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-set-color <color> <indices> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-change-type <index> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-make-color <color> <indices> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-make-aux <indices> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-change-mv <indices> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-advance-type <index> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-alternate-mv <startColor> <guide ax ay bx by color> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-alternate-mv-crossing <startColor> <guide ax ay bx by color> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-draw-crease <fold|aux> <segment ax ay bx by color> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-draw-symmetric <axis ax ay bx by color> <preselected> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-draw-point <index> <targetX> <targetY> <selectionDistance> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-draw <centerX> <centerY> <radiusX> <radiusY>");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-draw-free <centerX> <centerY> <radiusX> <radiusY>");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-three-point <ax> <ay> <bx> <by> <cx> <cy>");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-separate <centerX> <centerY> <ax> <ay> <bx> <by>");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-concentric <circle x y r color> <ax> <ay> <bx> <by>");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-concentric-select <candidateIndex> <target x y r color> <ref1 x y r color> <ref2 x y r color>");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-concentric-two <circle1 x y r color> <circle2 x y r color>");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-invert-circle <subject x y r color> <inversion x y r color>");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-invert-line <segment ax ay bx by color> <inversion x y r color>");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-organize <circleCount> [x y r color]... <lineCount> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-change-color <circleIndices> <auxLineIndices> <r> <g> <b> <circleCount> [x y r color]... <lineCount> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-tangent-point <pointX> <pointY> <circle x y r color> <lineCount> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-circle-tangent-two <circle1 x y r color> <circle2 x y r color>");
        System.err.println("   or: OrieditaGeometryOracle measure-length <ax> <ay> <bx> <by>");
        System.err.println("   or: OrieditaGeometryOracle measure-angle <ax> <ay> <centerX> <centerY> <bx> <by>");
        System.err.println("   or: OrieditaGeometryOracle custom-line-type <customType> <lineColor>");
        System.err.println("   or: OrieditaGeometryOracle orh-import-summary <path>");
        System.err.println("   or: OrieditaGeometryOracle orh-export-fixture");
        System.err.println("   or: OrieditaGeometryOracle obj-import-summary <path>");
        System.err.println("   or: OrieditaGeometryOracle dxf-export-fixture");
        System.exit(2);
    }
}
