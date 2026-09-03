def rw(p,f):
    s=open(p,encoding='utf-8').read(); n=f(s); assert n!=s,p; open(p,'w',encoding='utf-8',newline='\n').write(n)
def sub1(s,a,b):
    assert s.count(a)==1,(a[:70],s.count(a)); return s.replace(a,b)
F='vendor/servo/components/fonts/glyph.rs'; L='vendor/servo/components/layout/flow/inline/'
def glyph(s):
    s=sub1(s,'''    /// A [`ShapedTextSlice`] that is a word that ends with a white space glyphs.
    /// Typically whitespace glyphs are placed in a separate slice, but that may not be
    /// the case with `white-space: break-spaces`.
    WordAndWhiteSpace,''','''    /// A [`ShapedTextSlice`] that is a word that ends with a white space glyphs.
    /// Typically whitespace glyphs are placed in a separate slice, but that may not be
    /// the case with `white-space: break-spaces`.
    WordAndWhiteSpace,
    /// A continuation piece of a word that was split for `overflow-wrap: anywhere` /
    /// `break-word`. The opportunity before it is only taken when the whole word cannot
    /// fit on a line by itself.
    WordFragment,''')
    s=sub1(s,'''        match self.slice_type {
            ShapedTextSliceType::Word => false,
            ShapedTextSliceType::WhiteSpace | ShapedTextSliceType::WordAndWhiteSpace => true,
        }
    }''','''        match self.slice_type {
            ShapedTextSliceType::Word | ShapedTextSliceType::WordFragment => false,
            ShapedTextSliceType::WhiteSpace | ShapedTextSliceType::WordAndWhiteSpace => true,
        }
    }

    /// Whether this slice continues a word split for emergency (`overflow-wrap`) breaking.
    #[inline]
    pub fn is_word_fragment(&self) -> bool {
        matches!(self.slice_type, ShapedTextSliceType::WordFragment)
    }''')
    return s
rw(F,glyph)
def sq(s):
    s=sub1(s,'''            // Push the non-whitespace part of the range.
            if !slice.is_empty() {
                current_character_offset += self.text[slice].chars().count();
                maybe_push_run(
                    self.slicer
                        .slice_until_character_offset(current_character_offset, slice_type),
                );
            }''','''            // Push the non-whitespace part of the range. When the style allows breaking
            // inside words, emit one slice per character so line layout can take an
            // emergency break when the whole word cannot fit on a line.
            if !slice.is_empty() {
                let word = &self.text[slice];
                let count = word.chars().count();
                if can_break_anywhere && count > 1 {
                    for index in 0..count {
                        current_character_offset += 1;
                        let piece_type = match index {
                            0 => ShapedTextSliceType::Word,
                            i if i + 1 == count => match slice_type {
                                ShapedTextSliceType::WordAndWhiteSpace => slice_type,
                                _ => ShapedTextSliceType::WordFragment,
                            },
                            _ => ShapedTextSliceType::WordFragment,
                        };
                        maybe_push_run(
                            self.slicer
                                .slice_until_character_offset(current_character_offset, piece_type),
                        );
                    }
                } else {
                    current_character_offset += count;
                    maybe_push_run(
                        self.slicer
                            .slice_until_character_offset(current_character_offset, slice_type),
                    );
                }
            }''')
    return s
rw(L+'shaping_queue.rs',sq)
def tr(s):
    s=sub1(s,'''        let mut character_range_start = self.character_range.start;
        for (run_index, run) in self.runs.iter().enumerate() {
            let new_character_range_end = character_range_start + run.character_count();

            // Break before each unbreakable run in this TextRun, except the first unless the
            // linebreaker was set to break before the first run.
            if run_index != 0 || soft_wrap_policy == SegmentStartSoftWrapPolicy::Force {
                ifc.process_soft_wrap_opportunity();
            }
''','''        // Inline size of the whole word each run belongs to, counting the run and the
        // `WordFragment` runs that continue it. Used to decide emergency breaks.
        let mut word_sizes = vec![Au::zero(); self.runs.len()];
        let mut accumulated = Au::zero();
        for (index, run) in self.runs.iter().enumerate().rev() {
            accumulated = run.total_advance() +
                match self.runs.get(index + 1).is_some_and(|next| next.is_word_fragment()) {
                    true => accumulated,
                    false => Au::zero(),
                };
            word_sizes[index] = accumulated;
        }
        let mut character_range_start = self.character_range.start;
        for (run_index, run) in self.runs.iter().enumerate() {
            let new_character_range_end = character_range_start + run.character_count();

            // Break before each unbreakable run in this TextRun, except the first unless the
            // linebreaker was set to break before the first run.
            if run.is_word_fragment() {
                let word_size = word_sizes[..run_index]
                    .iter()
                    .zip(self.runs.iter())
                    .rev()
                    .find(|(_, run)| !run.is_word_fragment())
                    .map(|(size, _)| *size)
                    .unwrap_or(word_sizes[run_index]);
                ifc.process_weak_wrap_opportunity(word_size, run.total_advance());
            } else if run_index != 0 || soft_wrap_policy == SegmentStartSoftWrapPolicy::Force {
                ifc.process_soft_wrap_opportunity();
            }
''')
    if 'use app_units::Au;' not in s:
        s=sub1(s,'use std::ops::Range;','use std::ops::Range;\n\nuse app_units::Au;')
    return s
rw(L+'text_run.rs',tr)
def m(s):
    s=sub1(s,'''    /// Commit the current unbrekable segment to the current line. In addition, this will
    /// place all floats in the unbreakable segment and expand the line dimensions.
    fn commit_current_segment_to_line(&mut self) {''','''    /// Process a wrap opportunity inside a word split for `overflow-wrap` / `word-break:
    /// break-word`. Words that fit on an empty line stay unbreakable; words wider than the
    /// line are broken wherever the next piece would overflow the current line.
    fn process_weak_wrap_opportunity(&mut self, word_inline_size: Au, next_advance: Au) {
        if self.current_line_segment.line_items.is_empty() {
            return;
        }
        if self.text_wrap_mode == TextWrapMode::Nowrap {
            return;
        }
        let line_inline_size = self.containing_block().size.inline;
        if word_inline_size <= line_inline_size {
            return;
        }
        let remaining = line_inline_size - self.current_line.inline_position;
        if self.current_line_segment.inline_size + next_advance > remaining {
            self.process_soft_wrap_opportunity();
        }
    }

    /// Commit the current unbrekable segment to the current line. In addition, this will
    /// place all floats in the unbreakable segment and expand the line dimensions.
    fn commit_current_segment_to_line(&mut self) {''')
    s=sub1(s,'''        for (run_index, run) in segment.runs.iter().enumerate() {
            // Break before each unbreakable run in this TextRun, except the first unless the
            // linebreaker was set to break before the first run.
            if can_wrap && (run_index != 0 || break_at_start) {
                self.line_break_opportunity();
            }''','''        for (run_index, run) in segment.runs.iter().enumerate() {
            // Break before each unbreakable run in this TextRun, except the first unless the
            // linebreaker was set to break before the first run. Word fragments only break
            // in emergencies, which do not contribute to min-content.
            if can_wrap && (run_index != 0 || break_at_start) && !run.is_word_fragment() {
                self.line_break_opportunity();
            }''')
    return s
rw(L+'mod.rs',m)
print('wrap patched')
