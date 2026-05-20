package origami.crease_pattern;

import origami.crease_pattern.element.LineColor;
import origami.crease_pattern.element.Point;

public class PointSet {
    public int getNumLines() {
        return 0;
    }

    public Point getPoint(int index) {
        return new Point();
    }

    public int getBegin(int index) {
        return 0;
    }

    public int getEnd(int index) {
        return 0;
    }

    public LineColor getColor(int index) {
        return LineColor.BLACK_0;
    }
}
