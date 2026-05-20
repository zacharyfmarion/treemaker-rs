package oriedita.editor.databinding;

import java.awt.Color;
import java.io.Serializable;

public class FoldedFigureModel implements Serializable {
    private Color frontColor = new Color(255, 255, 50);
    private Color backColor = new Color(233, 233, 233);
    private Color lineColor = Color.black;

    public void set(FoldedFigureModel model) {
        frontColor = model.frontColor;
        backColor = model.backColor;
        lineColor = model.lineColor;
    }

    public Color getFrontColor() { return frontColor; }
    public void setFrontColor(Color value) { frontColor = value; }
    public Color getBackColor() { return backColor; }
    public void setBackColor(Color value) { backColor = value; }
    public Color getLineColor() { return lineColor; }
    public void setLineColor(Color value) { lineColor = value; }
}
