//! Parallel-coordinates projection (Slice 10 Layer F) — the zero-JS
//! alternative to interactive 3D for high-dimensional data (Inselberg; used
//! by Apache ECharts for multivariate analysis). Pure function, integer math
//! only (INV-01 spirit: no floating point in derived output values), fully
//! derivable: same input → same SVG.

/// One account's series across ordered axes (e.g. months).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelSeries {
    /// Entity label (account id) — rendered in each polyline's `<title>`.
    pub label: String,
    /// Ordered (axis label, value) pairs; axes must be identical across
    /// series for a meaningful projection.
    pub values: Vec<(String, i64)>,
}

/// Render a parallel-coordinates SVG from integer series.
///
/// Normalization is per-axis min-max using **integer arithmetic**:
/// `y = top + span * (v - min) / (max - min)`; a degenerate axis (max == min)
/// places every point at the vertical midline. Each polyline carries a
/// `<title>` (WCAG: accessible name). Empty input renders an empty string.
pub fn render_parallel_coordinates(
    series: &[ParallelSeries],
    width: usize,
    height: usize,
) -> String {
    if series.is_empty() {
        return String::new();
    }
    // Axes come from the first series; series with mismatched axes are
    // skipped (defensive: callers sort consistently).
    let axes: Vec<String> = series[0].values.iter().map(|(a, _)| a.clone()).collect();
    if axes.is_empty() {
        return String::new();
    }

    let left = 60usize;
    let right = width.saturating_sub(20);
    let top = 30usize;
    let bottom = height.saturating_sub(40);
    let span = bottom.saturating_sub(top);
    let axis_span = if axes.len() > 1 {
        (right - left) / (axes.len() - 1)
    } else {
        0
    };

    // Per-axis min/max over all series (integer).
    let mut mins: Vec<i64> = vec![i64::MAX; axes.len()];
    let mut maxs: Vec<i64> = vec![i64::MIN; axes.len()];
    for s in series {
        for (idx, (_, v)) in s.values.iter().enumerate() {
            if idx < mins.len() {
                mins[idx] = mins[idx].min(*v);
                maxs[idx] = maxs[idx].max(*v);
            }
        }
    }

    let axis_x = |idx: usize| -> usize { left + idx * axis_span };

    // Integer normalization: v in [min,max] -> y in [top,bottom].
    let axis_y = |idx: usize, v: i64| -> usize {
        let min = mins[idx];
        let max = maxs[idx];
        if max == min {
            return top + span / 2;
        }
        let numerator = span * (v - min) as usize;
        top + numerator / ((max - min) as usize)
    };

    let mut svg = format!(
        "<svg viewBox=\"0 0 {width} {height}\" width=\"100%\" height=\"{height}\" role=\"img\" \
         aria-label=\"科目多維平行座標投影\" xmlns=\"http://www.w3.org/2000/svg\">\
         <title>科目多維平行座標投影</title>\
         <desc>每條折線代表一個科目，跨越各月度軸，位置經整數正規化。</desc>"
    );

    // Axis lines + labels.
    for (idx, axis) in axes.iter().enumerate() {
        let x = axis_x(idx);
        svg.push_str(&format!(
            "<line x1=\"{x}\" y1=\"{top}\" x2=\"{x}\" y2=\"{bottom}\" stroke=\"#e0e0e0\"></line>"
        ));
        svg.push_str(&format!(
            "<text x=\"{x}\" y=\"{}\" font-size=\"10\" text-anchor=\"middle\" fill=\"#666666\">{axis}</text>",
            bottom + 16
        ));
    }

    // One polyline per account.
    for s in series {
        if s.values.len() != axes.len() {
            continue;
        }
        let points: Vec<String> = s
            .values
            .iter()
            .enumerate()
            .map(|(idx, (_, v))| format!("{},{}", axis_x(idx), axis_y(idx, *v)))
            .collect();
        svg.push_str(&format!(
            "<polyline points=\"{}\" fill=\"none\" stroke=\"#005fcc\" stroke-width=\"1.5\">\
             <title>{}</title></polyline>",
            points.join(" "),
            s.label
        ));
    }

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<ParallelSeries> {
        vec![
            ParallelSeries {
                label: "ACC-A".to_string(),
                values: vec![
                    ("2026-09".to_string(), 1_000),
                    ("2026-10".to_string(), 2_000),
                    ("2026-11".to_string(), 3_000),
                ],
            },
            ParallelSeries {
                label: "ACC-B".to_string(),
                values: vec![
                    ("2026-09".to_string(), 3_000),
                    ("2026-10".to_string(), 2_000),
                    ("2026-11".to_string(), 1_000),
                ],
            },
        ]
    }

    #[test]
    fn empty_series_renders_empty_string() {
        assert_eq!(render_parallel_coordinates(&[], 600, 200), "");
    }

    #[test]
    fn each_account_gets_one_polyline_with_accessible_title() {
        let svg = render_parallel_coordinates(&sample(), 600, 200);
        // PAR-001: one polyline per account, each with an accessible title.
        assert_eq!(svg.matches("<polyline").count(), 2);
        assert!(svg.contains("<title>ACC-A</title>"));
        assert!(svg.contains("<title>ACC-B</title>"));
        assert!(svg.contains("aria-label"), "SVG must be labelled");
    }

    #[test]
    fn axes_are_drawn_with_labels() {
        let svg = render_parallel_coordinates(&sample(), 600, 200);
        assert_eq!(svg.matches("<line").count(), 3, "one axis per month");
        assert!(svg.contains(">2026-09<"));
        assert!(svg.contains(">2026-11<"));
    }

    #[test]
    fn integer_normalization_places_extremes_on_axis_bounds() {
        // ACC-A spans min..max on every axis: first axis min -> top-most
        // polyline point for ACC-A is at the axis top (y == top) and ACC-B at
        // the bottom (y == bottom) on that axis.
        let svg = render_parallel_coordinates(&sample(), 600, 200);
        // top=30, bottom=height-40=160.
        assert!(
            svg.contains("points=\"60,30 320,95 580,160\""),
            "ACC-A spans the full axis range (min at top, max at bottom)"
        );
        assert!(
            svg.contains("points=\"60,160 320,95 580,30\""),
            "ACC-B mirrors ACC-A"
        );
    }

    #[test]
    fn degenerate_axis_places_points_at_midline() {
        let series = vec![ParallelSeries {
            label: "ACC-FLAT".to_string(),
            values: vec![("2026-09".to_string(), 500), ("2026-10".to_string(), 500)],
        }];
        let svg = render_parallel_coordinates(&series, 600, 200);
        // top=30, span=130, midline = 30 + 65 = 95.
        assert!(
            svg.contains("60,95"),
            "degenerate axis must use the midline"
        );
    }
}
