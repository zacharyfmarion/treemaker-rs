import origami.crease_pattern.CustomLineTypes;
import origami.crease_pattern.LineSegmentSet;
import origami.crease_pattern.OritaCalc;
import origami.crease_pattern.element.Circle;
import origami.crease_pattern.element.LineColor;
import origami.crease_pattern.element.LineSegment;
import origami.crease_pattern.element.Point;
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

public class OrieditaGeometryOracle {
    public static void main(String[] args) throws Exception {
        if (args.length < 1) {
            usage("missing command");
        }

        switch (args[0]) {
            case "intersection" -> intersection(args);
            case "intersect-divide" -> intersectDivide(args);
            case "intersect-divide-pair" -> intersectDividePair(args);
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
        System.err.println("   or: OrieditaGeometryOracle custom-line-type <customType> <lineColor>");
        System.err.println("   or: OrieditaGeometryOracle orh-import-summary <path>");
        System.err.println("   or: OrieditaGeometryOracle orh-export-fixture");
        System.err.println("   or: OrieditaGeometryOracle obj-import-summary <path>");
        System.err.println("   or: OrieditaGeometryOracle dxf-export-fixture");
        System.exit(2);
    }
}
