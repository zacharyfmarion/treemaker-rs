//! Direct Rust port of TreeMaker 5.0.1's distributable `tmNLCO_alm`.

pub const ERROR_TOO_MANY_ITERATIONS: i32 = 1;

const WEIGHT_START: f64 = 10.0;
const WEIGHT_RATIO: f64 = 10.0;
const WEIGHT_MAX: f64 = 1.0e8;
const TOL_FEAS: f64 = 1.0e-5;
const TOL_F: f64 = 1.0e-5;
const ITER_OUTER_MAX: usize = 50;
const ITER_INNER_MAX: usize = 200;
const TOL_G: f64 = 1.0e-5;
const ALF: f64 = 1.0e-4;
const DEGREES: f64 = 0.017453292519943296;

pub trait DifferentiableFn {
    fn func(&self, x: &[f64]) -> f64;
    fn grad(&self, x: &[f64], grad: &mut [f64]);
}

pub struct OneVarFn {
    ix: usize,
    a: f64,
    b: f64,
}

impl OneVarFn {
    pub fn new(ix: usize, a: f64, b: f64) -> Self {
        Self { ix, a, b }
    }
}

impl DifferentiableFn for OneVarFn {
    fn func(&self, x: &[f64]) -> f64 {
        self.a * x[self.ix] + self.b
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = self.a;
    }
}

pub struct TwoVarFn {
    ix: usize,
    a: f64,
    iy: usize,
    b: f64,
    c: f64,
}

impl TwoVarFn {
    pub fn new(ix: usize, a: f64, iy: usize, b: f64, c: f64) -> Self {
        Self { ix, a, iy, b, c }
    }
}

impl DifferentiableFn for TwoVarFn {
    fn func(&self, x: &[f64]) -> f64 {
        self.a * x[self.ix] + self.b * x[self.iy] + self.c
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = self.a;
        grad[self.iy] = self.b;
    }
}

pub struct PathFn1 {
    ix: usize,
    iy: usize,
    jx: usize,
    jy: usize,
    lij: f64,
}

impl PathFn1 {
    pub fn new(ix: usize, iy: usize, jx: usize, jy: usize, lij: f64) -> Self {
        Self {
            ix,
            iy,
            jx,
            jy,
            lij,
        }
    }
}

impl DifferentiableFn for PathFn1 {
    fn func(&self, x: &[f64]) -> f64 {
        x[0] * self.lij - (sqr(x[self.ix] - x[self.jx]) + sqr(x[self.iy] - x[self.jy])).sqrt()
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[0] = self.lij;
        let mut temp = (sqr(x[self.ix] - x[self.jx]) + sqr(x[self.iy] - x[self.jy])).sqrt();
        if temp != 0.0 {
            temp = 1.0 / temp;
        }
        grad[self.ix] = temp * (x[self.jx] - x[self.ix]);
        grad[self.jx] = -grad[self.ix];
        grad[self.iy] = temp * (x[self.jy] - x[self.iy]);
        grad[self.jy] = -grad[self.iy];
    }
}

pub struct PathAngleFn1 {
    ix: usize,
    iy: usize,
    jx: usize,
    jy: usize,
    ca: f64,
    sa: f64,
}

impl PathAngleFn1 {
    pub fn new(ix: usize, iy: usize, jx: usize, jy: usize, angle: f64) -> Self {
        let radians = angle * DEGREES;
        Self {
            ix,
            iy,
            jx,
            jy,
            ca: radians.cos(),
            sa: radians.sin(),
        }
    }
}

impl DifferentiableFn for PathAngleFn1 {
    fn func(&self, x: &[f64]) -> f64 {
        (x[self.ix] - x[self.jx]) * self.sa + (x[self.jy] - x[self.iy]) * self.ca
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = self.sa;
        grad[self.jx] = -self.sa;
        grad[self.iy] = -self.ca;
        grad[self.jy] = self.ca;
    }
}

pub struct PathAngleFn2 {
    ix: usize,
    iy: usize,
    vx: f64,
    vy: f64,
    ca: f64,
    sa: f64,
}

impl PathAngleFn2 {
    pub fn new(ix: usize, iy: usize, vx: f64, vy: f64, angle: f64) -> Self {
        let radians = angle * DEGREES;
        Self {
            ix,
            iy,
            vx,
            vy,
            ca: radians.cos(),
            sa: radians.sin(),
        }
    }
}

impl DifferentiableFn for PathAngleFn2 {
    fn func(&self, x: &[f64]) -> f64 {
        (x[self.ix] - self.vx) * self.sa + (self.vy - x[self.iy]) * self.ca
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = self.sa;
        grad[self.iy] = -self.ca;
    }
}

pub struct StrainPathFn1 {
    ix: usize,
    iy: usize,
    jx: usize,
    jy: usize,
    lfix: f64,
    lvar: f64,
}

impl StrainPathFn1 {
    pub fn new(ix: usize, iy: usize, jx: usize, jy: usize, lfix: f64, lvar: f64) -> Self {
        Self {
            ix,
            iy,
            jx,
            jy,
            lfix,
            lvar,
        }
    }
}

impl DifferentiableFn for StrainPathFn1 {
    fn func(&self, x: &[f64]) -> f64 {
        self.lfix + x[0] * self.lvar
            - (sqr(x[self.ix] - x[self.jx]) + sqr(x[self.iy] - x[self.jy])).sqrt()
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[0] = self.lvar;
        let mut temp = (sqr(x[self.ix] - x[self.jx]) + sqr(x[self.iy] - x[self.jy])).sqrt();
        if temp != 0.0 {
            temp = 1.0 / temp;
        }
        grad[self.ix] = temp * (x[self.jx] - x[self.ix]);
        grad[self.jx] = -grad[self.ix];
        grad[self.iy] = temp * (x[self.jy] - x[self.iy]);
        grad[self.jy] = -grad[self.iy];
    }
}

pub struct StrainPathFn2 {
    ix: usize,
    iy: usize,
    vx: f64,
    vy: f64,
    lfix: f64,
    lvar: f64,
}

impl StrainPathFn2 {
    pub fn new(ix: usize, iy: usize, vx: f64, vy: f64, lfix: f64, lvar: f64) -> Self {
        Self {
            ix,
            iy,
            vx,
            vy,
            lfix,
            lvar,
        }
    }
}

impl DifferentiableFn for StrainPathFn2 {
    fn func(&self, x: &[f64]) -> f64 {
        self.lfix + x[0] * self.lvar
            - (sqr(x[self.ix] - self.vx) + sqr(x[self.iy] - self.vy)).sqrt()
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[0] = self.lvar;
        let mut temp = (sqr(x[self.ix] - self.vx) + sqr(x[self.iy] - self.vy)).sqrt();
        if temp != 0.0 {
            temp = 1.0 / temp;
        }
        grad[self.ix] = temp * (self.vx - x[self.ix]);
        grad[self.iy] = temp * (self.vy - x[self.iy]);
    }
}

pub struct StrainPathFn3 {
    ux: f64,
    uy: f64,
    vx: f64,
    vy: f64,
    lfix: f64,
    lvar: f64,
}

impl StrainPathFn3 {
    pub fn new(ux: f64, uy: f64, vx: f64, vy: f64, lfix: f64, lvar: f64) -> Self {
        Self {
            ux,
            uy,
            vx,
            vy,
            lfix,
            lvar,
        }
    }
}

impl DifferentiableFn for StrainPathFn3 {
    fn func(&self, x: &[f64]) -> f64 {
        self.lfix + x[0] * self.lvar - (sqr(self.ux - self.vx) + sqr(self.uy - self.vy)).sqrt()
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[0] = self.lvar;
    }
}

pub struct StickToEdgeFn {
    ix: usize,
    iy: usize,
    w: f64,
    h: f64,
}

impl StickToEdgeFn {
    pub fn new(ix: usize, iy: usize, w: f64, h: f64) -> Self {
        Self { ix, iy, w, h }
    }
}

impl DifferentiableFn for StickToEdgeFn {
    fn func(&self, x: &[f64]) -> f64 {
        10.0 * x[self.ix] * (x[self.ix] - self.w) * x[self.iy] * (x[self.iy] - self.h)
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = 10.0 * (2.0 * x[self.ix] - self.w) * x[self.iy] * (x[self.iy] - self.h);
        grad[self.iy] = 10.0 * (2.0 * x[self.iy] - self.h) * x[self.ix] * (x[self.ix] - self.w);
    }
}

pub struct StickToLineFn {
    ix: usize,
    iy: usize,
    px: f64,
    py: f64,
    ca: f64,
    sa: f64,
}

impl StickToLineFn {
    pub fn new(ix: usize, iy: usize, px: f64, py: f64, angle: f64) -> Self {
        let radians = angle * DEGREES;
        Self {
            ix,
            iy,
            px,
            py,
            ca: radians.cos(),
            sa: radians.sin(),
        }
    }
}

impl DifferentiableFn for StickToLineFn {
    fn func(&self, x: &[f64]) -> f64 {
        (-x[self.ix] + self.px) * self.sa + (x[self.iy] - self.py) * self.ca
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = -self.sa;
        grad[self.iy] = self.ca;
    }
}

pub struct PairFn1A {
    ix: usize,
    iy: usize,
    jx: usize,
    jy: usize,
    ca: f64,
    sa: f64,
}

impl PairFn1A {
    pub fn new(ix: usize, iy: usize, jx: usize, jy: usize, angle: f64) -> Self {
        let radians = angle * DEGREES;
        Self {
            ix,
            iy,
            jx,
            jy,
            ca: radians.cos(),
            sa: radians.sin(),
        }
    }
}

impl DifferentiableFn for PairFn1A {
    fn func(&self, x: &[f64]) -> f64 {
        (x[self.ix] - x[self.jx]) * self.ca + (x[self.iy] - x[self.jy]) * self.sa
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = self.ca;
        grad[self.iy] = self.sa;
        grad[self.jx] = -self.ca;
        grad[self.jy] = -self.sa;
    }
}

pub struct PairFn1B {
    ix: usize,
    iy: usize,
    jx: usize,
    jy: usize,
    px: f64,
    py: f64,
    ca: f64,
    sa: f64,
}

impl PairFn1B {
    pub fn new(ix: usize, iy: usize, jx: usize, jy: usize, px: f64, py: f64, angle: f64) -> Self {
        let radians = angle * DEGREES;
        Self {
            ix,
            iy,
            jx,
            jy,
            px,
            py,
            ca: radians.cos(),
            sa: radians.sin(),
        }
    }
}

impl DifferentiableFn for PairFn1B {
    fn func(&self, x: &[f64]) -> f64 {
        (-x[self.ix] - x[self.jx] + 2.0 * self.px) * self.sa
            + (x[self.iy] + x[self.jy] - 2.0 * self.py) * self.ca
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = -self.sa;
        grad[self.iy] = self.ca;
        grad[self.jx] = -self.sa;
        grad[self.jy] = self.ca;
    }
}

pub struct PairFn2A {
    ix: usize,
    iy: usize,
    vx: f64,
    vy: f64,
    ca: f64,
    sa: f64,
}

impl PairFn2A {
    pub fn new(ix: usize, iy: usize, vx: f64, vy: f64, angle: f64) -> Self {
        let radians = angle * DEGREES;
        Self {
            ix,
            iy,
            vx,
            vy,
            ca: radians.cos(),
            sa: radians.sin(),
        }
    }
}

impl DifferentiableFn for PairFn2A {
    fn func(&self, x: &[f64]) -> f64 {
        (x[self.ix] - self.vx) * self.ca + (x[self.iy] - self.vy) * self.sa
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = self.ca;
        grad[self.iy] = self.sa;
    }
}

pub struct PairFn2B {
    ix: usize,
    iy: usize,
    vx: f64,
    vy: f64,
    px: f64,
    py: f64,
    ca: f64,
    sa: f64,
}

impl PairFn2B {
    pub fn new(ix: usize, iy: usize, vx: f64, vy: f64, px: f64, py: f64, angle: f64) -> Self {
        let radians = angle * DEGREES;
        Self {
            ix,
            iy,
            vx,
            vy,
            px,
            py,
            ca: radians.cos(),
            sa: radians.sin(),
        }
    }
}

impl DifferentiableFn for PairFn2B {
    fn func(&self, x: &[f64]) -> f64 {
        (-x[self.ix] - self.vx + 2.0 * self.px) * self.sa
            + (x[self.iy] + self.vy - 2.0 * self.py) * self.ca
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = -self.sa;
        grad[self.iy] = self.ca;
    }
}

pub struct CollinearFn1 {
    ix: usize,
    iy: usize,
    jx: usize,
    jy: usize,
    kx: usize,
    ky: usize,
}

impl CollinearFn1 {
    pub fn new(ix: usize, iy: usize, jx: usize, jy: usize, kx: usize, ky: usize) -> Self {
        Self {
            ix,
            iy,
            jx,
            jy,
            kx,
            ky,
        }
    }
}

impl DifferentiableFn for CollinearFn1 {
    fn func(&self, x: &[f64]) -> f64 {
        (x[self.jy] - x[self.iy]) * (x[self.kx] - x[self.jx])
            - (x[self.ky] - x[self.jy]) * (x[self.jx] - x[self.ix])
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = x[self.ky] - x[self.jy];
        grad[self.kx] = x[self.jy] - x[self.iy];
        grad[self.jx] = -(grad[self.ix] + grad[self.kx]);
        grad[self.iy] = x[self.jx] - x[self.kx];
        grad[self.ky] = x[self.ix] - x[self.jx];
        grad[self.jy] = -(grad[self.iy] + grad[self.ky]);
    }
}

pub struct CollinearFn2 {
    ix: usize,
    iy: usize,
    jx: usize,
    jy: usize,
    wx: f64,
    wy: f64,
}

impl CollinearFn2 {
    pub fn new(ix: usize, iy: usize, jx: usize, jy: usize, wx: f64, wy: f64) -> Self {
        Self {
            ix,
            iy,
            jx,
            jy,
            wx,
            wy,
        }
    }
}

impl DifferentiableFn for CollinearFn2 {
    fn func(&self, x: &[f64]) -> f64 {
        (x[self.jy] - x[self.iy]) * (self.wx - x[self.jx])
            - (self.wy - x[self.jy]) * (x[self.jx] - x[self.ix])
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = self.wy - x[self.jy];
        grad[self.jx] = -grad[self.ix];
        grad[self.iy] = x[self.jx] - self.wx;
        grad[self.jy] = -grad[self.iy];
    }
}

pub struct CollinearFn3 {
    ix: usize,
    iy: usize,
    vx: f64,
    vy: f64,
    wx: f64,
    wy: f64,
}

impl CollinearFn3 {
    pub fn new(ix: usize, iy: usize, vx: f64, vy: f64, wx: f64, wy: f64) -> Self {
        Self {
            ix,
            iy,
            vx,
            vy,
            wx,
            wy,
        }
    }
}

impl DifferentiableFn for CollinearFn3 {
    fn func(&self, x: &[f64]) -> f64 {
        (self.vy - x[self.iy]) * (self.wx - self.vx) - (self.wy - self.vy) * (self.vx - x[self.ix])
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = self.wy - self.vy;
        grad[self.iy] = self.vx - self.wx;
    }
}

pub struct QuantizeAngleFn1 {
    ix: usize,
    iy: usize,
    jx: usize,
    jy: usize,
    n: usize,
    oa: f64,
    da: f64,
    wt: f64,
}

impl QuantizeAngleFn1 {
    pub fn new(ix: usize, iy: usize, jx: usize, jy: usize, n: usize, offset: f64) -> Self {
        Self {
            ix,
            iy,
            jx,
            jy,
            n,
            oa: offset * DEGREES,
            da: 180.0 * DEGREES / n as f64,
            wt: 2.0_f64.powi(n as i32 - 1),
        }
    }
}

impl DifferentiableFn for QuantizeAngleFn1 {
    fn func(&self, x: &[f64]) -> f64 {
        let r2 = sqr(x[self.ix] - x[self.jx]) + sqr(x[self.iy] - x[self.jy]);
        let f1 = r2.powf(-0.5 * self.n as f64);

        let mut f2 = 1.0;
        for k in 0..self.n {
            let ak = k as f64 * self.da - self.oa;
            let fk = (x[self.ix] - x[self.jx]) * ak.sin() - (x[self.iy] - x[self.jy]) * ak.cos();
            f2 *= fk;
        }

        self.wt * f1 * f2
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);

        let r2 = sqr(x[self.ix] - x[self.jx]) + sqr(x[self.iy] - x[self.jy]);
        let f1 = r2.powf(-0.5 * self.n as f64);

        let mut f2 = 1.0;
        for k in 0..self.n {
            let ak = k as f64 * self.da - self.oa;
            let fk = (x[self.ix] - x[self.jx]) * ak.sin() - (x[self.iy] - x[self.jy]) * ak.cos();
            f2 *= fk;
        }

        let f3 = -f1 * self.n as f64 * f2 / r2;
        let mut temp = f3 * (x[self.ix] - x[self.jx]);
        grad[self.ix] += temp;
        grad[self.jx] -= temp;
        temp = f3 * (x[self.iy] - x[self.jy]);
        grad[self.iy] += temp;
        grad[self.jy] -= temp;

        for l in 0..self.n {
            let mut dl = 1.0;
            for k in 0..self.n {
                if k == l {
                    continue;
                }
                let ak = k as f64 * self.da - self.oa;
                dl *= (x[self.ix] - x[self.jx]) * ak.sin() - (x[self.iy] - x[self.jy]) * ak.cos();
            }
            let al = l as f64 * self.da - self.oa;
            temp = f1 * al.sin() * dl;
            grad[self.ix] += temp;
            grad[self.jx] -= temp;
            temp = f1 * -al.cos() * dl;
            grad[self.iy] += temp;
            grad[self.jy] -= temp;
        }

        for value in grad.iter_mut() {
            *value *= self.wt;
        }
    }
}

pub struct QuantizeAngleFn2 {
    ix: usize,
    iy: usize,
    vx: f64,
    vy: f64,
    n: usize,
    oa: f64,
    da: f64,
    wt: f64,
}

impl QuantizeAngleFn2 {
    pub fn new(ix: usize, iy: usize, vx: f64, vy: f64, n: usize, offset: f64) -> Self {
        let oa = offset * DEGREES;
        let da = 180.0 * DEGREES / n as f64;
        let mut wt = 3.0_f64.powi(n as i32 - 1);
        for _ in 1..n {
            let ak = oa + da;
            wt /= ak.sin();
        }
        Self {
            ix,
            iy,
            vx,
            vy,
            n,
            oa,
            da,
            wt,
        }
    }
}

impl DifferentiableFn for QuantizeAngleFn2 {
    fn func(&self, x: &[f64]) -> f64 {
        let mut fret = 1.0;
        for k in 0..self.n {
            let ak = self.oa + k as f64 * self.da;
            let fk = (x[self.ix] - self.vx) * ak.sin() - (x[self.iy] - self.vy) * ak.cos();
            fret *= fk;
        }
        self.wt * fret
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        for k in 0..self.n {
            let ak = self.oa + k as f64 * self.da;
            let mut dk = 1.0;
            for l in 0..self.n {
                if k == l {
                    continue;
                }
                let al = self.oa + l as f64 * self.da;
                let fl = (x[self.ix] - self.vx) * al.sin() - (x[self.iy] - self.vy) * al.cos();
                dk *= fl;
            }
            let mut temp = ak.sin() * dk;
            grad[self.ix] += temp;
            temp = -ak.cos() * dk;
            grad[self.iy] += temp;
        }
        for value in grad.iter_mut() {
            *value *= self.wt;
        }
    }
}

pub struct MultiStrainPathFn1 {
    ix: usize,
    iy: usize,
    jx: usize,
    jy: usize,
    lfix: f64,
    vi: Vec<usize>,
    vf: Vec<f64>,
}

impl MultiStrainPathFn1 {
    pub fn new(
        ix: usize,
        iy: usize,
        jx: usize,
        jy: usize,
        lfix: f64,
        vi: Vec<usize>,
        vf: Vec<f64>,
    ) -> Self {
        Self {
            ix,
            iy,
            jx,
            jy,
            lfix,
            vi,
            vf,
        }
    }
}

impl DifferentiableFn for MultiStrainPathFn1 {
    fn func(&self, x: &[f64]) -> f64 {
        let mut pathlen = self.lfix;
        for (index, coeff) in self.vi.iter().zip(&self.vf) {
            pathlen += x[*index] * coeff;
        }
        pathlen - (sqr(x[self.ix] - x[self.jx]) + sqr(x[self.iy] - x[self.jy])).sqrt()
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        for (index, coeff) in self.vi.iter().zip(&self.vf) {
            grad[*index] = *coeff;
        }
        let mut temp = (sqr(x[self.ix] - x[self.jx]) + sqr(x[self.iy] - x[self.jy])).sqrt();
        if temp != 0.0 {
            temp = 1.0 / temp;
        }
        grad[self.ix] = temp * (x[self.jx] - x[self.ix]);
        grad[self.jx] = -grad[self.ix];
        grad[self.iy] = temp * (x[self.jy] - x[self.iy]);
        grad[self.jy] = -grad[self.iy];
    }
}

pub struct MultiStrainPathFn2 {
    ix: usize,
    iy: usize,
    vx: f64,
    vy: f64,
    lfix: f64,
    vi: Vec<usize>,
    vf: Vec<f64>,
}

impl MultiStrainPathFn2 {
    pub fn new(
        ix: usize,
        iy: usize,
        vx: f64,
        vy: f64,
        lfix: f64,
        vi: Vec<usize>,
        vf: Vec<f64>,
    ) -> Self {
        Self {
            ix,
            iy,
            vx,
            vy,
            lfix,
            vi,
            vf,
        }
    }
}

impl DifferentiableFn for MultiStrainPathFn2 {
    fn func(&self, x: &[f64]) -> f64 {
        let mut pathlen = self.lfix;
        for (index, coeff) in self.vi.iter().zip(&self.vf) {
            pathlen += x[*index] * coeff;
        }
        pathlen - (sqr(x[self.ix] - self.vx) + sqr(x[self.iy] - self.vy)).sqrt()
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        for (index, coeff) in self.vi.iter().zip(&self.vf) {
            grad[*index] = *coeff;
        }
        let mut temp = (sqr(x[self.ix] - self.vx) + sqr(x[self.iy] - self.vy)).sqrt();
        if temp != 0.0 {
            temp = 1.0 / temp;
        }
        // Preserve TreeMaker 5.0.1 behavior: the y-coordinate derivative is
        // omitted in MultiStrainPathFn2::Grad.
        grad[self.ix] = temp * (self.vx - x[self.ix]);
    }
}

pub struct MultiStrainPathFn3 {
    ux: f64,
    uy: f64,
    vx: f64,
    vy: f64,
    lfix: f64,
    vi: Vec<usize>,
    vf: Vec<f64>,
}

impl MultiStrainPathFn3 {
    pub fn new(
        ux: f64,
        uy: f64,
        vx: f64,
        vy: f64,
        lfix: f64,
        vi: Vec<usize>,
        vf: Vec<f64>,
    ) -> Self {
        Self {
            ux,
            uy,
            vx,
            vy,
            lfix,
            vi,
            vf,
        }
    }
}

impl DifferentiableFn for MultiStrainPathFn3 {
    fn func(&self, x: &[f64]) -> f64 {
        let mut pathlen = self.lfix;
        for (index, coeff) in self.vi.iter().zip(&self.vf) {
            pathlen += x[*index] * coeff;
        }
        pathlen - (sqr(self.ux - self.vx) + sqr(self.uy - self.vy)).sqrt()
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        for (index, coeff) in self.vi.iter().zip(&self.vf) {
            grad[*index] = *coeff;
        }
    }
}

pub struct CornerFn {
    ix: usize,
    w: f64,
}

impl CornerFn {
    pub fn new(ix: usize, w: f64) -> Self {
        Self { ix, w }
    }
}

impl DifferentiableFn for CornerFn {
    fn func(&self, x: &[f64]) -> f64 {
        x[self.ix] * (x[self.ix] - self.w)
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[self.ix] = 2.0 * x[self.ix] - self.w;
    }
}

pub struct NlcoAlm {
    size: usize,
    lower_bounds: Vec<f64>,
    upper_bounds: Vec<f64>,
    num_bounds: usize,
    lag_mul: Vec<f64>,
    weight: f64,
    objective: Option<Box<dyn DifferentiableFn>>,
    equalities: Vec<Box<dyn DifferentiableFn>>,
    inequalities: Vec<Box<dyn DifferentiableFn>>,
    max_step: f64,
}

impl NlcoAlm {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            lower_bounds: Vec::new(),
            upper_bounds: Vec::new(),
            num_bounds: 0,
            lag_mul: Vec::new(),
            weight: 0.0,
            objective: None,
            equalities: Vec::new(),
            inequalities: Vec::new(),
            max_step: 0.0,
        }
    }

    pub fn set_objective(&mut self, objective: Box<dyn DifferentiableFn>) {
        assert!(self.objective.is_none());
        self.objective = Some(objective);
    }

    pub fn add_linear_equality(&mut self, constraint: Box<dyn DifferentiableFn>) {
        self.equalities.push(constraint);
    }

    pub fn add_nonlinear_equality(&mut self, constraint: Box<dyn DifferentiableFn>) {
        self.equalities.push(constraint);
    }

    pub fn add_linear_inequality(&mut self, constraint: Box<dyn DifferentiableFn>) {
        self.inequalities.push(constraint);
    }

    pub fn add_nonlinear_inequality(&mut self, constraint: Box<dyn DifferentiableFn>) {
        self.inequalities.push(constraint);
    }

    pub fn set_bounds(&mut self, lower_bounds: Vec<f64>, upper_bounds: Vec<f64>) {
        assert_eq!(lower_bounds.len(), upper_bounds.len());
        assert_eq!(lower_bounds.len(), self.size);
        self.lower_bounds = lower_bounds;
        self.upper_bounds = upper_bounds;
        self.num_bounds = self.lower_bounds.len();
    }

    pub fn minimize(&mut self, x: &mut [f64]) -> i32 {
        assert_ne!(self.size, 0);
        assert_eq!(x.len(), self.size);
        assert!(self.objective.is_some());

        let ne = self.equalities.len();
        let ni = self.inequalities.len();
        self.lag_mul = vec![0.0; ne + ni + 2 * self.num_bounds];

        let mut sum = 0.0;
        for i in 0..self.num_bounds {
            sum += sqr(self.upper_bounds[i] - self.lower_bounds[i]);
        }
        self.max_step = sum.sqrt();
        if self.max_step == 0.0 {
            self.max_step = 1.0;
        }

        let mut iter_outer = 1;
        self.weight = WEIGHT_START;
        let mut fval_old = 1.0e30;

        while iter_outer < ITER_OUTER_MAX {
            let mut iter_inner = 0;
            let mut f_alm = 0.0;
            self.minimize_aug_lag(x, &mut iter_inner, &mut f_alm);

            let mut feas = 0.0_f64;
            for i in 0..ne {
                let f = self.equalities[i].func(x);
                feas = max(feas, f.abs());
                self.lag_mul[i] += 2.0 * self.weight * f;
            }
            for i in 0..ni {
                let f = self.inequalities[i].func(x);
                if f > 0.0 {
                    feas = max(feas, f);
                }
                let lm = self.lag_mul[i + ne];
                let mu = -0.5 * lm / self.weight;
                self.lag_mul[i + ne] = if f < mu {
                    0.0
                } else {
                    lm + 2.0 * self.weight * f
                };
            }
            for (i, x_i) in x.iter().enumerate().take(self.num_bounds) {
                let f = self.lower_bounds[i] - *x_i;
                if f > 0.0 {
                    feas = max(feas, f);
                }
                let idx = i + ne + ni;
                let lm = self.lag_mul[idx];
                let mu = -0.5 * lm / self.weight;
                self.lag_mul[idx] = if f < mu {
                    0.0
                } else {
                    lm + 2.0 * self.weight * f
                };
            }
            for (i, x_i) in x.iter().enumerate().take(self.num_bounds) {
                let f = *x_i - self.upper_bounds[i];
                if f > 0.0 {
                    feas = max(feas, f);
                }
                let idx = i + ne + ni + self.num_bounds;
                let lm = self.lag_mul[idx];
                let mu = -0.5 * lm / self.weight;
                self.lag_mul[idx] = if f < mu {
                    0.0
                } else {
                    lm + 2.0 * self.weight * f
                };
            }

            let fval = self.objective().func(x);
            if feas < TOL_FEAS {
                if (fval - fval_old).abs() < TOL_F {
                    return 0;
                }
                fval_old = fval;
            }

            self.weight *= WEIGHT_RATIO;
            if self.weight > WEIGHT_MAX {
                self.weight = WEIGHT_MAX;
            }

            iter_outer += 1;
        }

        ERROR_TOO_MANY_ITERATIONS
    }

    fn minimize_aug_lag(&mut self, x: &mut [f64], iter_inner: &mut usize, f_min: &mut f64) {
        *f_min = self.aug_lag_fn(x);
        let mut g = vec![0.0; self.size];
        self.aug_lag_grad(x, &mut g);

        let mut hess_inv = vec![vec![0.0; self.size]; self.size];
        let mut srch_dir = vec![0.0; self.size];
        for i in 0..self.size {
            hess_inv[i][i] = 1.0;
            srch_dir[i] = -g[i];
        }

        let mut x_new = vec![0.0; self.size];
        let mut dg = vec![0.0; self.size];
        let mut hdg = vec![0.0; self.size];

        for iter in 1..=ITER_INNER_MAX {
            *iter_inner = iter;
            let f_old = *f_min;
            self.line_search_aug_lag(x, f_old, &g, &mut srch_dir, &mut x_new, f_min);

            for i in 0..self.size {
                srch_dir[i] = x_new[i] - x[i];
                x[i] = x_new[i];
            }

            let mut xtest = 0.0;
            for i in 0..self.size {
                let xtemp = srch_dir[i].abs() / max(x[i].abs(), 1.0);
                if xtemp > xtest {
                    xtest = xtemp;
                }
            }
            if xtest < 4.0 * f64::EPSILON {
                return;
            }

            dg[..self.size].copy_from_slice(&g[..self.size]);
            self.aug_lag_grad(x, &mut g);

            let mut gtest = 0.0;
            let den = max(*f_min, 1.0);
            for i in 0..self.size {
                let gtemp = g[i].abs() * max(x[i].abs(), 1.0) / den;
                if gtemp > gtest {
                    gtest = gtemp;
                }
            }
            if gtest < TOL_G {
                return;
            }

            for i in 0..self.size {
                dg[i] = g[i] - dg[i];
            }
            for i in 0..self.size {
                hdg[i] = 0.0;
                for (j, dg_j) in dg.iter().enumerate().take(self.size) {
                    hdg[i] += hess_inv[i][j] * dg_j;
                }
            }

            let mut fac = 0.0;
            let mut fae = 0.0;
            let mut sumdg = 0.0;
            let mut sumxi = 0.0;
            for i in 0..self.size {
                fac += dg[i] * srch_dir[i];
                fae += dg[i] * hdg[i];
                sumdg += sqr(dg[i]);
                sumxi += sqr(srch_dir[i]);
            }

            if fac > (f64::EPSILON * sumdg * sumxi).sqrt() {
                fac = 1.0 / fac;
                let fad = 1.0 / fae;
                for i in 0..self.size {
                    dg[i] = fac * srch_dir[i] - fad * hdg[i];
                }
                for i in 0..self.size {
                    for j in i..self.size {
                        hess_inv[i][j] += fac * srch_dir[i] * srch_dir[j] - fad * hdg[i] * hdg[j]
                            + fae * dg[i] * dg[j];
                        hess_inv[j][i] = hess_inv[i][j];
                    }
                }
            }

            for i in 0..self.size {
                srch_dir[i] = 0.0;
                for (j, g_j) in g.iter().enumerate().take(self.size) {
                    srch_dir[i] -= hess_inv[i][j] * g_j;
                }
            }
        }
    }

    fn line_search_aug_lag(
        &self,
        x_old: &[f64],
        f_old: f64,
        g_old: &[f64],
        srch_dir: &mut [f64],
        x_new: &mut [f64],
        f_new: &mut f64,
    ) {
        let mut dir_mag = 0.0;
        for value in srch_dir.iter().take(self.size) {
            dir_mag += sqr(*value);
        }
        dir_mag = dir_mag.sqrt();
        if dir_mag > self.max_step {
            for value in srch_dir.iter_mut().take(self.size) {
                *value *= self.max_step / dir_mag;
            }
        }

        let mut slope = 0.0;
        for i in 0..self.size {
            slope += g_old[i] * srch_dir[i];
        }
        if slope >= 0.0 {
            return;
        }

        let mut lmtest = 0.0;
        for i in 0..self.size {
            let lmtemp = srch_dir[i].abs() / max(x_old[i].abs(), 1.0);
            if lmtest < lmtemp {
                lmtest = lmtemp;
            }
        }
        let lm_min = f64::EPSILON / lmtest;

        let mut lm = 1.0;
        let mut lm_2 = 0.0;
        let mut f_new_2 = 0.0;
        loop {
            for i in 0..self.size {
                x_new[i] = x_old[i] + lm * srch_dir[i];
            }
            *f_new = self.aug_lag_fn(x_new);

            if lm < lm_min {
                x_new[..self.size].copy_from_slice(&x_old[..self.size]);
                return;
            }

            let f_tobeat = f_old + ALF * lm * slope;
            if *f_new <= f_tobeat {
                return;
            }

            let lm_tmp = if lm == 1.0 {
                -slope / (2.0 * (*f_new - f_old - slope))
            } else {
                let rhs1 = *f_new - f_old - lm * slope;
                let rhs2 = f_new_2 - f_old - lm_2 * slope;
                let lmsqr = sqr(lm);
                let lmsqr2 = sqr(lm_2);
                let lmd = lm - lm_2;
                let a = (rhs1 / lmsqr - rhs2 / lmsqr2) / lmd;
                let b = (-lm_2 * rhs1 / lmsqr + lm * rhs2 / lmsqr2) / lmd;
                let mut lm_tmp = if a == 0.0 {
                    -slope / (2.0 * b)
                } else {
                    let discr = sqr(b) - 3.0 * a * slope;
                    if discr < 0.0 {
                        0.5 * lm
                    } else if b <= 0.0 {
                        (-b + discr.sqrt()) / (3.0 * a)
                    } else {
                        -slope / (b + discr.sqrt())
                    }
                };
                if lm_tmp > 0.5 * lm {
                    lm_tmp = 0.5 * lm;
                }
                lm_tmp
            };
            lm_2 = lm;
            f_new_2 = *f_new;
            lm = max(lm_tmp, 0.1 * lm);
        }
    }

    fn aug_lag_fn(&self, x: &[f64]) -> f64 {
        let ne = self.equalities.len();
        let ni = self.inequalities.len();
        let mut fret = self.objective().func(x);

        for i in 0..ne {
            let lm = self.lag_mul[i];
            let f = self.equalities[i].func(x);
            fret += (lm + f * self.weight) * f;
        }
        for i in 0..ni {
            let lm = self.lag_mul[i + ne];
            let f = self.inequalities[i].func(x);
            let mu = -0.5 * lm / self.weight;
            fret += if f < mu {
                mu
            } else {
                (lm + f * self.weight) * f
            };
        }
        for (i, x_i) in x.iter().enumerate().take(self.num_bounds) {
            let lm = self.lag_mul[i + ne + ni];
            let f = self.lower_bounds[i] - *x_i;
            let mu = -0.5 * lm / self.weight;
            fret += if f < mu {
                mu
            } else {
                (lm + f * self.weight) * f
            };
        }
        for (i, x_i) in x.iter().enumerate().take(self.num_bounds) {
            let lm = self.lag_mul[i + ne + ni + self.num_bounds];
            let f = *x_i - self.upper_bounds[i];
            let mu = -0.5 * lm / self.weight;
            fret += if f < mu {
                mu
            } else {
                (lm + f * self.weight) * f
            };
        }

        fret
    }

    fn aug_lag_grad(&self, x: &[f64], g: &mut [f64]) {
        let ne = self.equalities.len();
        let ni = self.inequalities.len();
        let tol_lm = 4.0 * f64::EPSILON;
        let mut gscr = vec![0.0; self.size];

        self.objective().grad(x, g);

        for i in 0..ne {
            let lm = self.lag_mul[i];
            let f = self.equalities[i].func(x);
            let gmul = lm + 2.0 * f * self.weight;
            if gmul.abs() > tol_lm {
                self.equalities[i].grad(x, &mut gscr);
                for j in 0..self.size {
                    g[j] += gmul * gscr[j];
                }
            }
        }
        for i in 0..ni {
            let lm = self.lag_mul[i + ne];
            let f = self.inequalities[i].func(x);
            let mu = -0.5 * lm / self.weight;
            if f >= mu {
                let gmul = lm + 2.0 * f * self.weight;
                if gmul.abs() > tol_lm {
                    self.inequalities[i].grad(x, &mut gscr);
                    for j in 0..self.size {
                        g[j] += gmul * gscr[j];
                    }
                }
            }
        }
        for i in 0..self.num_bounds {
            let lm = self.lag_mul[i + ne + ni];
            let f = self.lower_bounds[i] - x[i];
            let mu = -0.5 * lm / self.weight;
            if f >= mu {
                let gmul = lm + 2.0 * f * self.weight;
                if gmul.abs() > tol_lm {
                    g[i] -= gmul;
                }
            }
        }
        for i in 0..self.num_bounds {
            let lm = self.lag_mul[i + ne + ni + self.num_bounds];
            let f = x[i] - self.upper_bounds[i];
            let mu = -0.5 * lm / self.weight;
            if f >= mu {
                let gmul = lm + 2.0 * f * self.weight;
                if gmul.abs() > tol_lm {
                    g[i] += gmul;
                }
            }
        }
    }

    fn objective(&self) -> &dyn DifferentiableFn {
        self.objective
            .as_deref()
            .expect("NlcoAlm objective must be set before minimizing")
    }
}

fn max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

fn sqr(value: f64) -> f64 {
    value * value
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Quadratic;

    impl DifferentiableFn for Quadratic {
        fn func(&self, x: &[f64]) -> f64 {
            sqr(x[0] - 2.0) + sqr(x[1] + 1.0)
        }

        fn grad(&self, x: &[f64], grad: &mut [f64]) {
            grad[0] = 2.0 * (x[0] - 2.0);
            grad[1] = 2.0 * (x[1] + 1.0);
        }
    }

    #[test]
    fn minimizes_unconstrained_quadratic() {
        let mut optimizer = NlcoAlm::new(2);
        optimizer.set_objective(Box::new(Quadratic));
        let mut x = vec![8.0, 5.0];

        assert_eq!(optimizer.minimize(&mut x), 0);
        assert!((x[0] - 2.0).abs() < 1.0e-5, "{x:?}");
        assert!((x[1] + 1.0).abs() < 1.0e-5, "{x:?}");
    }

    #[test]
    fn respects_bounds_and_equality() {
        let mut optimizer = NlcoAlm::new(2);
        optimizer.set_objective(Box::new(Quadratic));
        optimizer.add_linear_equality(Box::new(OneVarFn::new(1, 1.0, -0.25)));
        optimizer.set_bounds(vec![0.0, -10.0], vec![1.0, 10.0]);
        let mut x = vec![8.0, 5.0];

        assert_eq!(optimizer.minimize(&mut x), 0);
        assert!((x[0] - 1.0).abs() < 1.0e-4, "{x:?}");
        assert!((x[1] - 0.25).abs() < 1.0e-4, "{x:?}");
    }
}
