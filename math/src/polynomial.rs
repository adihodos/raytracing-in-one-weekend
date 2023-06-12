use num::Float;

pub fn poly_linear<T: Float>(a: T, b: T, roots: &mut [T]) -> u32 {
    if a.is_zero() {
        return 0;
    }
    roots[0] = -b / a;
    1
}

pub fn poly_quadratic<T: Float + num::FromPrimitive>(a: T, b: T, c: T, roots: &mut [T]) -> u32 {
    if a.is_zero() {
        //
        // solve linear equation
        return poly_linear(b, c, roots);
    }

    let delta = b * b - T::from_i32(4).unwrap() * a * c;
    if delta.is_zero() {
        roots[0] = -b / T::from_i32(2).unwrap() * a;
        roots[1] = -b / T::from_i32(2).unwrap() * a;
        return 2;
    }

    if delta.is_sign_positive() {
        let q = -(b + b.signum() * delta.sqrt()) / T::from_i32(2).unwrap();
        roots[0] = q / a;
        roots[1] = c / q;

        if roots[0] > roots[1] {
            roots.swap(0, 1);
        }
        return 2;
    }

    //
    // no real value roots
    return 0;
}

pub fn poly_roots_cubic<T: Float + num::FromPrimitive>(
    s: T,
    p: T,
    q: T,
    r: T,
    roots: &mut [T],
) -> u32 {
    //
    // See this : http://www.cs.iastate.edu/~cs577/handouts/polyroots.pdf
    // for a more detailed explanation.

    if s.is_zero() {
        //
        // solve quadratic equation
        return poly_quadratic(p, q, r, roots);
    }

    //
    // Divide by s to get equation in standard form :
    // x^3 + (p/s)x^2 + (q/s)x + (r/s) = 0
    let p = p / s;
    let q = q / s;
    let r = r / s;

    //
    // Reduce equation to the normal form (y^3 + ay + b) = 0 by
    // substituting x = y - p/3
    // a = 1/3 * (3q - p^2)
    // b = 1/27 * (2p^3 -9pq + 27r)
    let one_third = T::from_i32(3).unwrap().recip();
    let one_twentyseven = T::from_i32(27).unwrap().recip();
    let offset = one_third * p;
    let a = q - p * offset;
    let b =
        r + p * (T::from_i32(2).unwrap() * p * p - T::from_i32(9).unwrap() * q) * one_twentyseven;
    let half_b = b * T::from_f32(0.5f32).unwrap();

    let delta = half_b * half_b + a * a * a * one_twentyseven;
    let delta = if delta.is_zero() { T::zero() } else { delta };

    if delta > T::zero() {
        //
        // one root is real, the other two complex
        let sqrt_delta = delta.sqrt();
        let tmp = -half_b + sqrt_delta;
        if tmp.is_sign_positive() {
            roots[0] = tmp.powf(one_third);
        } else {
            roots[0] = -(-tmp).powf(one_third);
        }

        let tmp1 = -half_b - sqrt_delta;
        if tmp1.is_sign_positive() {
            roots[0] = roots[0] + tmp1.powf(one_third);
        } else {
            roots[0] = roots[0] - (-tmp1).powf(one_third);
        }

        roots[0] = roots[0] - offset;
        return 1;
    } else if delta < T::zero() {
        //
        // three distinct real roots
        let sqrt3 = T::from_i32(3).unwrap().sqrt();
        let const_fact = (-one_third * a).sqrt();
        let angle = one_third * (-delta).sqrt().atan2(-half_b);
        let (sns, css) = angle.sin_cos();

        roots[0] = T::from_i32(2).unwrap() * const_fact * css - offset;
        roots[1] = -const_fact * (css + sqrt3 * sns) - offset;
        roots[2] = -const_fact * (css - sqrt3 * sns) - offset;

        return 3;
    } else {
        //
        // three real roots, two of them equal
        let tmp = if half_b.is_sign_positive() {
            -(half_b.powf(one_third))
        } else {
            (-half_b).powf(one_third)
        };

        roots[0] = T::from_i32(2).unwrap() * tmp - offset;
        roots[1] = -tmp - offset;
        roots[2] = roots[1];

        return 3;
    }
}

pub fn poly_roots_quartic<T: Float + num::FromPrimitive>(
    t: T,
    s: T,
    p: T,
    q: T,
    r: T,
    roots: &mut [T],
) -> u32 {
    if t.is_zero() {
        //
        // solve cubic equation
        return poly_roots_cubic(s, p, q, r, roots);
    }

    //
    // make monic polynomial
    let s = s / t;
    let p = p / t;
    let q = q / t;
    let r = r / t;

    let p_pow2 = p * p;
    let one_quarter = T::from_f32(0.25f32).unwrap();

    //
    // compute the roots of the resolvent cubic
    let mut resolvent_roots: [T; 3] = [T::zero(); 3];

    let root_cnt = poly_roots_cubic(
        T::one(),
        -q,
        p * r - T::from_i32(4).unwrap() * s,
        T::from_i32(4).unwrap() * q * s - r * r - p_pow2 * s,
        &mut resolvent_roots,
    );

    assert!(root_cnt != 0);

    let z1 = resolvent_roots[0];
    let delta = one_quarter * p_pow2 - q + z1;

    if delta < T::zero() {
        //
        // no real solutions
        return 0;
    } else if delta > T::zero() {
        let r = delta.sqrt();
        let term_a = T::from_f32(0.75f32).unwrap() * p_pow2 - r * r - T::from_i32(2).unwrap() * q;
        let term_b = T::from_f32(0.25f32).unwrap()
            * (T::from_i32(4).unwrap() * p * q - T::from_i32(8).unwrap() * r - p_pow2 * p)
            * r.recip();

        let t_diff = term_a - term_b;
        let t_sum = term_a + term_b;

        let mut roots_num = 0usize;

        if t_sum.is_sign_positive() {
            let d = t_sum.sqrt();
            roots[roots_num] =
                -T::from_f32(0.25f32).unwrap() * p + T::from_f32(0.5f32).unwrap() * (r + d);
            roots_num += 1;
            roots[roots_num] =
                -T::from_f32(0.25f32).unwrap() * p + T::from_f32(0.5f32).unwrap() * (r - d);
            roots_num += 1;
        }

        if t_diff.is_sign_positive() {
            let e = t_diff.sqrt();
            roots[roots_num] =
                -T::from_f32(0.25f32).unwrap() * p - T::from_f32(0.5f32).unwrap() * (r + e);
            roots_num += 1;
            roots[roots_num] =
                -T::from_f32(0.25f32).unwrap() * p - T::from_f32(0.5f32).unwrap() * (r - e);
            roots_num += 1;
        }

        return roots_num as u32;
    } else {
        let first_sqr = z1 * z1 - T::from_i32(4).unwrap() * s;
        if first_sqr.is_sign_positive() {
            let term_a = T::from_f32(0.75f32).unwrap() * p_pow2 - T::from_i32(2).unwrap() * q;
            let term_b = T::from_i32(2).unwrap() * (first_sqr).sqrt();

            let t_sum = term_a + term_b;
            let t_diff = term_a - term_b;

            let mut roots_num = 0usize;

            if t_sum.is_sign_positive() {
                let d = t_sum.sqrt();
                roots[roots_num] =
                    -T::from_f32(0.25f32).unwrap() * p + T::from_f32(0.5f32).unwrap() * (r + d);
                roots_num += 1;
                roots[roots_num] =
                    -T::from_f32(0.25).unwrap() * p + T::from_f32(0.5).unwrap() * (r - d);
                roots_num += 1;
            }

            if t_diff.is_sign_positive() {
                let e = t_diff.sqrt();
                roots[roots_num] =
                    -T::from_f32(0.25).unwrap() * p - T::from_f32(0.5).unwrap() * (r + e);
                roots_num += 1;

                roots[roots_num] =
                    -T::from_f32(0.25).unwrap() * p - T::from_f32(0.5).unwrap() * (r - e);
                roots_num += 1;
            }
            return roots_num as u32;
        }
        return 0;
    }
}
