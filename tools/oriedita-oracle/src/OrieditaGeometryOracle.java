import origami.crease_pattern.OritaCalc;
import origami.crease_pattern.element.LineSegment;

public class OrieditaGeometryOracle {
    public static void main(String[] args) {
        if (args.length < 1) {
            usage("missing command");
        }

        switch (args[0]) {
            case "intersection" -> intersection(args);
            default -> usage("unknown command: " + args[0]);
        }
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

    private static double parse(String value) {
        return Double.parseDouble(value);
    }

    private static void usage(String message) {
        System.err.println(message);
        System.err.println("usage: OrieditaGeometryOracle intersection <strict|sweet> <default|precision> ax ay bx by cx cy dx dy");
        System.exit(2);
    }
}
