use std::fmt;

// One histogram
#[derive(Debug)]
pub struct Histogram {
	// offset of this histogram bins from the global histogram
	pub first: usize,

	// offset of the last element of the histogram. TODO required?
	pub last: usize,

	// total number of data points stored in the histogram
	pub num_points: u32,

	// histogram bins
	pub bins: Vec<f32>
}

impl Histogram {
	pub fn new(first: usize, last: usize, num_points: u32, bins: Vec<f32>) -> Histogram {
		assert_eq!(last-first+1, bins.len(), "histogram length does not match first/last.");
		Histogram {first, last, num_points, bins}
	}

	// Returns the value of a bin if the bin is present in this
	// histogram
	pub fn get_bin_count(&self, bin: usize) -> Option<f32> {
		if bin < self.first || bin > self.last {
			None
		} else {
			Some(self.bins[bin-self.first])
		}
	}
}

// a set of histograms
#[derive(Debug)]
pub struct Dataset {
	// number of histogram windows (number of simulations)
	pub num_windows: usize,

	// number of global histogram bins
	pub num_bins: usize,

	// min value of the histogram
	pub hist_min: f32,

	// max value of the histogram
	pub hist_max: f32,

	// width of a bin in unit of x
	pub bin_width: f32,

	// locations of biases
	pub bias_x0: Vec<f32>,

	// force constants of biases
	pub bias_fc: Vec<f32>,

	// value of kT
	pub kT: f32,

	// histogram for each window
	pub histograms: Vec<Histogram>,

	// flag for cyclic reaction coordinates
	pub cyclic: bool,
}

impl Dataset {
	
	pub fn new(num_bins: usize, bin_width: f32, hist_min: f32, hist_max: f32, bias_x0: Vec<f32>, bias_fc: Vec<f32>, kT: f32, histograms: Vec<Histogram>, cyclic: bool) -> Dataset {
		let num_windows = histograms.len();
		Dataset{num_windows, num_bins, bin_width, hist_min, hist_max, bias_x0, bias_fc, kT, histograms, cyclic}
	}

	
	// Harmonic bias calculation: bias = 0.5*k(dx)^2
	// if cyclic is true, lowest and highest bins are assumed to be
	// neighbors
	pub fn calc_bias(&self, bin: usize, window: usize) -> f32 {
		let x = self.get_x_for_bin(bin);
		let mut dx = (x-self.bias_x0[window]).abs();
		if self.cyclic {
			let hist_len = self.hist_max-self.hist_min;
			if dx > 0.5*hist_len {
				dx -= hist_len;
			}
		}
		0.5*self.bias_fc[window]*dx*dx
	}

	// get center x value for a bin 
	pub fn get_x_for_bin(&self, bin: usize) -> f32 {
		self.hist_min + self.bin_width * ((bin as f32) + 0.5)
	}

}

impl fmt::Display for Dataset {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut datapoints: u32 = 0;
		for h in &self.histograms {
			datapoints += h.num_points;
		}
		write!(f, "{} windows, {} datapoints", self.num_windows, datapoints)
    }
}

#[cfg(test)]
mod tests {
	use super::*;
	use super::super::k_B;

	fn build_hist() -> Histogram {
		Histogram::new(
			5, // first
			9, // last
			22, // num_points
			vec![1.0, 1.0, 3.0, 5.0, 12.0] // bins
		)
	}

	fn build_hist_set() -> Dataset {
		let h = build_hist();
		Dataset::new( 
			7, // num bins
			1.0, // bin width
			0.0, // hist min
			9.0, // hist max
			vec![7.5], // x0
			vec![10.0], // fc
			300.0*k_B, // kT
			vec![h], // hists
			false // cyclic
		)
	}

	#[test]
	fn get_bin_count() {
		let h = build_hist();
		let expected = vec![None, Some(1.0), Some(1.0), Some(3.0), Some(5.0), Some(12.0), None];
		let test_offset = 4;
		for i in 4..10 {
			match expected[i-test_offset] {
				Some(x) => assert_eq!(x, h.get_bin_count(i).unwrap()),
				None => assert!(h.get_bin_count(i) == None)
			}
		}
	}

	#[test]
	fn calc_bias() {
		let ds = build_hist_set();

		// 7th element -> x=7.5, x0=7.5
		assert_eq!(0.0, ds.calc_bias(7, 0));

		// 8th element -> x=8.5, x0=7.5
		assert_eq!(5.0, ds.calc_bias(8, 0));
		

		// 9th element -> x=9.5, x0=7.5
		assert_eq!(20.0, ds.calc_bias(9, 0));


		// 1st element -> x=0.5, x0=7.5. non-cyclic!
		assert_eq!(245.0, ds.calc_bias(0, 0));
	}

	#[test]
	fn calc_bias_offset_cyclic() {
		let mut ds = build_hist_set();
		ds.cyclic = true;

		// 7th element -> x=7.5, x0=7.5
		assert_eq!(0.0, ds.calc_bias(7, 0));

		// 8th element -> x=8.5, x0=7.5
		assert_eq!(5.0, ds.calc_bias(8, 0));
		

		// 9th element -> x=9.5, x0=7.5
		assert_eq!(20.0, ds.calc_bias(9, 0));


		// 1st element -> x=0.5, x0=7.5
		// cyclic flag makes bin 0 neighboring bin 9, so the distance is actually 2
		assert_eq!(20.0, ds.calc_bias(0, 0));
	}

	#[test]
	fn get_x_for_bin() {
		let ds = build_hist_set();
		let expected: Vec<f32> = vec![0,1,2,3,4,5,6,7,8].iter()
				.map(|x| *x as f32 + 0.5).collect(); 
		for i in 0..9 {
			assert_eq!(expected[i], ds.get_x_for_bin(i));
		}
	}
}