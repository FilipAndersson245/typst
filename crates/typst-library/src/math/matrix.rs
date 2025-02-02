use super::*;

const ROW_GAP: Em = Em::new(0.5);
const COL_GAP: Em = Em::new(0.5);
const VERTICAL_PADDING: Ratio = Ratio::new(0.1);

/// A column vector.
///
/// Content in the vector's elements can be aligned with the `&` symbol.
///
/// ## Example { #example }
/// ```example
/// $ vec(a, b, c) dot vec(1, 2, 3)
///     = a + 2b + 3c $
/// ```
///
/// Display: Vector
/// Category: math
#[element(LayoutMath)]
pub struct VecElem {
    /// The delimiter to use.
    ///
    /// ```example
    /// #set math.vec(delim: "[")
    /// $ vec(1, 2) $
    /// ```
    #[default(Some(Delimiter::Paren))]
    pub delim: Option<Delimiter>,

    /// The elements of the vector.
    #[variadic]
    pub children: Vec<Content>,
}

impl LayoutMath for VecElem {
    #[tracing::instrument(skip(ctx))]
    fn layout_math(&self, ctx: &mut MathContext) -> SourceResult<()> {
        let delim = self.delim(ctx.styles());
        let frame = layout_vec_body(ctx, &self.children(), Align::Center)?;
        layout_delimiters(
            ctx,
            frame,
            delim.map(Delimiter::open),
            delim.map(Delimiter::close),
            self.span(),
        )
    }
}

/// A matrix.
///
/// The elements of a row should be separated by commas, while the rows
/// themselves should be separated by semicolons. The semicolon syntax merges
/// preceding arguments separated by commas into an array. You can also use this
/// special syntax of math function calls to define custom functions that take
/// 2D data.
///
/// Content in cells that are in the same row can be aligned with the `&` symbol.
///
/// ## Example { #example }
/// ```example
/// $ mat(
///   1, 2, ..., 10;
///   2, 2, ..., 10;
///   dots.v, dots.v, dots.down, dots.v;
///   10, 10, ..., 10;
/// ) $
/// ```
///
/// Display: Matrix
/// Category: math
#[element(LayoutMath)]
pub struct MatElem {
    /// The delimiter to use.
    ///
    /// ```example
    /// #set math.mat(delim: "[")
    /// $ mat(1, 2; 3, 4) $
    /// ```
    #[default(Some(Delimiter::Paren))]
    pub delim: Option<Delimiter>,

    /// An array of arrays with the rows of the matrix.
    ///
    /// ```example
    /// #let data = ((1, 2, 3), (4, 5, 6))
    /// #let matrix = math.mat(..data)
    /// $ v := matrix $
    /// ```
    #[variadic]
    #[parse(
        let mut rows = vec![];
        let mut width = 0;

        let values = args.all::<Spanned<Value>>()?;
        if values.iter().any(|spanned| matches!(spanned.v, Value::Array(_))) {
            for Spanned { v, span } in values {
                let array = v.cast::<Array>().at(span)?;
                let row: Vec<_> = array.into_iter().map(Value::display).collect();
                width = width.max(row.len());
                rows.push(row);
            }
        } else {
            rows = vec![values.into_iter().map(|spanned| spanned.v.display()).collect()];
        }

        for row in &mut rows {
            if row.len() < width {
                row.resize(width, Content::empty());
            }
        }

        rows
    )]
    pub rows: Vec<Vec<Content>>,
}

impl LayoutMath for MatElem {
    #[tracing::instrument(skip(ctx))]
    fn layout_math(&self, ctx: &mut MathContext) -> SourceResult<()> {
        let delim = self.delim(ctx.styles());
        let frame = layout_mat_body(ctx, &self.rows())?;
        layout_delimiters(
            ctx,
            frame,
            delim.map(Delimiter::open),
            delim.map(Delimiter::close),
            self.span(),
        )
    }
}

/// A case distinction.
///
/// Content across different branches can be aligned with the `&` symbol.
///
/// ## Example { #example }
/// ```example
/// $ f(x, y) := cases(
///   1 "if" (x dot y)/2 <= 0,
///   2 "if" x "is even",
///   3 "if" x in NN,
///   4 "else",
/// ) $
/// ```
///
/// Display: Cases
/// Category: math
#[element(LayoutMath)]
pub struct CasesElem {
    /// The delimiter to use.
    ///
    /// ```example
    /// #set math.cases(delim: "[")
    /// $ x = cases(1, 2) $
    /// ```
    #[default(Delimiter::Brace)]
    pub delim: Delimiter,

    /// The branches of the case distinction.
    #[variadic]
    pub children: Vec<Content>,
}

impl LayoutMath for CasesElem {
    #[tracing::instrument(skip(ctx))]
    fn layout_math(&self, ctx: &mut MathContext) -> SourceResult<()> {
        let delim = self.delim(ctx.styles());
        let frame = layout_vec_body(ctx, &self.children(), Align::Left)?;
        layout_delimiters(ctx, frame, Some(delim.open()), None, self.span())
    }
}

/// A vector / matrix delimiter.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Cast)]
pub enum Delimiter {
    /// Delimit with parentheses.
    #[string("(")]
    Paren,
    /// Delimit with brackets.
    #[string("[")]
    Bracket,
    /// Delimit with curly braces.
    #[string("{")]
    Brace,
    /// Delimit with vertical bars.
    #[string("|")]
    Bar,
    /// Delimit with double vertical bars.
    #[string("||")]
    DoubleBar,
}

impl Delimiter {
    /// The delimiter's opening character.
    fn open(self) -> char {
        match self {
            Self::Paren => '(',
            Self::Bracket => '[',
            Self::Brace => '{',
            Self::Bar => '|',
            Self::DoubleBar => '‖',
        }
    }

    /// The delimiter's closing character.
    fn close(self) -> char {
        match self {
            Self::Paren => ')',
            Self::Bracket => ']',
            Self::Brace => '}',
            Self::Bar => '|',
            Self::DoubleBar => '‖',
        }
    }
}

/// Layout the inner contents of a vector.
fn layout_vec_body(
    ctx: &mut MathContext,
    column: &[Content],
    align: Align,
) -> SourceResult<Frame> {
    let gap = ROW_GAP.scaled(ctx);
    ctx.style(ctx.style.for_denominator());
    let mut flat = vec![];
    for child in column {
        flat.push(ctx.layout_row(child)?);
    }
    ctx.unstyle();
    Ok(stack(ctx, flat, align, gap, 0))
}

/// Layout the inner contents of a matrix.
fn layout_mat_body(ctx: &mut MathContext, rows: &[Vec<Content>]) -> SourceResult<Frame> {
    let row_gap = ROW_GAP.scaled(ctx);
    let col_gap = COL_GAP.scaled(ctx);

    let ncols = rows.first().map_or(0, |row| row.len());
    let nrows = rows.len();
    if ncols == 0 || nrows == 0 {
        return Ok(Frame::new(Size::zero()));
    }

    let mut heights = vec![(Abs::zero(), Abs::zero()); nrows];

    ctx.style(ctx.style.for_denominator());
    let mut cols = vec![vec![]; ncols];
    for (row, (ascent, descent)) in rows.iter().zip(&mut heights) {
        for (cell, col) in row.iter().zip(&mut cols) {
            let cell = ctx.layout_row(cell)?;
            ascent.set_max(cell.ascent());
            descent.set_max(cell.descent());
            col.push(cell);
        }
    }
    ctx.unstyle();

    let mut frame = Frame::new(Size::new(
        Abs::zero(),
        heights.iter().map(|&(a, b)| a + b).sum::<Abs>() + row_gap * (nrows - 1) as f64,
    ));
    let mut x = Abs::zero();
    for col in cols {
        let AlignmentResult { points, width: rcol } = alignments(&col);
        let mut y = Abs::zero();
        for (cell, &(ascent, descent)) in col.into_iter().zip(&heights) {
            let cell = cell.into_aligned_frame(ctx, &points, Align::Center);
            let pos = Point::new(
                if points.is_empty() { x + (rcol - cell.width()) / 2.0 } else { x },
                y + ascent - cell.ascent(),
            );
            frame.push_frame(pos, cell);
            y += ascent + descent + row_gap;
        }
        x += rcol + col_gap;
    }
    frame.size_mut().x = x - col_gap;

    Ok(frame)
}

/// Layout the outer wrapper around a vector's or matrices' body.
fn layout_delimiters(
    ctx: &mut MathContext,
    mut frame: Frame,
    left: Option<char>,
    right: Option<char>,
    span: Span,
) -> SourceResult<()> {
    let axis = scaled!(ctx, axis_height);
    let short_fall = DELIM_SHORT_FALL.scaled(ctx);
    let height = frame.height();
    let target = height + VERTICAL_PADDING.of(height);
    frame.set_baseline(height / 2.0 + axis);

    if let Some(left) = left {
        let mut left =
            GlyphFragment::new(ctx, left, span).stretch_vertical(ctx, target, short_fall);
        left.center_on_axis(ctx);
        ctx.push(left);
    }

    ctx.push(FrameFragment::new(ctx, frame));

    if let Some(right) = right {
        let mut right = GlyphFragment::new(ctx, right, span)
            .stretch_vertical(ctx, target, short_fall);
        right.center_on_axis(ctx);
        ctx.push(right);
    }

    Ok(())
}
