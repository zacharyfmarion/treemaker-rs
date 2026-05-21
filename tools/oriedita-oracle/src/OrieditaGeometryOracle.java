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
import origami.crease_pattern.worker.foldlineset.BranchTrim;
import origami.crease_pattern.worker.foldlineset.Fix1;
import origami.crease_pattern.worker.foldlineset.Fix2;
import origami.crease_pattern.worker.linesegmentset.IntersectDivide;

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
            case "foldline-branch-trim" -> foldLineBranchTrim(args);
            case "foldline-del-v" -> foldLineDelV(args);
            case "foldline-del-v-cc" -> foldLineDelVCc(args);
            case "foldline-del-v-pair" -> foldLineDelVPair(args);
            case "foldline-del-v-all" -> foldLineDelVAll(args);
            case "foldline-del-v-all-cc" -> foldLineDelVAllCc(args);
            case "foldline-fix1" -> foldLineFix1(args);
            case "foldline-fix2" -> foldLineFix2(args);
            case "foldline-set-color" -> foldLineSetColor(args);
            case "foldline-change-mv" -> foldLineChangeMv(args);
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
            case "foldline-divide-count" -> foldLineDivideCount(args);
            case "foldline-divide-ratio" -> foldLineDivideRatio(args);
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
        System.err.println("   or: OrieditaGeometryOracle foldline-branch-trim <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-del-v <px> <py> <snapRadius> <vertexRadius> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-del-v-cc <px> <py> <snapRadius> <vertexRadius> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-del-v-pair <i> <j> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-del-v-all <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-del-v-all-cc <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-fix1 <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-fix2 <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-set-color <color> <indices> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle foldline-change-mv <indices> <count> [ax ay bx by color]...");
        System.err.println("   or: OrieditaGeometryOracle custom-line-type <customType> <lineColor>");
        System.err.println("   or: OrieditaGeometryOracle orh-import-summary <path>");
        System.err.println("   or: OrieditaGeometryOracle orh-export-fixture");
        System.err.println("   or: OrieditaGeometryOracle obj-import-summary <path>");
        System.err.println("   or: OrieditaGeometryOracle dxf-export-fixture");
        System.exit(2);
    }
}
