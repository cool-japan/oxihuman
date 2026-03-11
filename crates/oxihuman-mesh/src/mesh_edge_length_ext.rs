// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Extended edge length analysis with percentiles and histograms.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeLengthExtStats {
    pub min: f32, pub max: f32, pub mean: f32, pub median: f32,
    pub p25: f32, pub p75: f32, pub std_dev: f32, pub count: usize,
}

#[allow(dead_code)]
pub fn edge_length_v(a:[f32;3],b:[f32;3]) -> f32 { let d=[b[0]-a[0],b[1]-a[1],b[2]-a[2]]; (d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt() }

#[allow(dead_code)]
pub fn collect_edge_lengths(positions:&[[f32;3]], indices:&[[u32;3]]) -> Vec<f32> {
    use std::collections::HashSet;
    let mut seen = HashSet::new(); let mut lengths = Vec::new();
    for tri in indices { for k in 0..3 { let a=tri[k]; let b=tri[(k+1)%3]; let key=if a<b{(a,b)}else{(b,a)};
        if seen.insert(key) { lengths.push(edge_length_v(positions[a as usize],positions[b as usize])); } } }
    lengths
}

#[allow(dead_code)]
pub fn percentile(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() { return 0.0; }
    let idx = ((sorted.len() as f32 - 1.0) * p.clamp(0.0, 1.0)) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

#[allow(dead_code)]
pub fn compute_edge_length_ext_stats(positions:&[[f32;3]], indices:&[[u32;3]]) -> EdgeLengthExtStats {
    let mut lengths = collect_edge_lengths(positions, indices);
    if lengths.is_empty() { return EdgeLengthExtStats { min:0.0,max:0.0,mean:0.0,median:0.0,p25:0.0,p75:0.0,std_dev:0.0,count:0 }; }
    lengths.sort_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = lengths.len(); let sum: f32 = lengths.iter().sum(); let mean = sum / n as f32;
    let var: f32 = lengths.iter().map(|&l| (l-mean)*(l-mean)).sum::<f32>() / n as f32;
    EdgeLengthExtStats {
        min: lengths[0], max: lengths[n-1], mean, median: percentile(&lengths, 0.5),
        p25: percentile(&lengths, 0.25), p75: percentile(&lengths, 0.75), std_dev: var.sqrt(), count: n,
    }
}

#[allow(dead_code)]
pub fn edge_length_histogram(positions:&[[f32;3]], indices:&[[u32;3]], bins:usize) -> Vec<usize> {
    let lengths = collect_edge_lengths(positions, indices);
    if lengths.is_empty() || bins == 0 { return vec![0; bins.max(1)]; }
    let mn = lengths.iter().copied().fold(f32::MAX, f32::min);
    let mx = lengths.iter().copied().fold(0.0f32, f32::max);
    let range = (mx - mn).max(1e-12);
    let mut hist = vec![0usize; bins];
    for l in &lengths { let b = (((l-mn)/range)*(bins as f32 -1.0)) as usize; hist[b.min(bins-1)]+=1; }
    hist
}

#[allow(dead_code)]
pub fn edge_length_ext_to_json(s:&EdgeLengthExtStats) -> String {
    format!("{{\"min\":{:.4},\"max\":{:.4},\"mean\":{:.4},\"median\":{:.4},\"count\":{}}}", s.min, s.max, s.mean, s.median, s.count)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tri() -> (Vec<[f32;3]>,Vec<[u32;3]>) { (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]], vec![[0,1,2]]) }
    #[test] fn test_edge_length_v() { assert!((edge_length_v([0.0,0.0,0.0],[3.0,4.0,0.0])-5.0).abs()<1e-5); }
    #[test] fn test_collect() { let(p,i)=tri(); let l=collect_edge_lengths(&p,&i); assert_eq!(l.len(),3); }
    #[test] fn test_percentile() { assert!((percentile(&[1.0,2.0,3.0,4.0],0.5)-2.0).abs()<1e-6); }
    #[test] fn test_stats() { let(p,i)=tri(); let s=compute_edge_length_ext_stats(&p,&i); assert!(s.mean>0.0); }
    #[test] fn test_median() { let(p,i)=tri(); let s=compute_edge_length_ext_stats(&p,&i); assert!(s.median>0.0); }
    #[test] fn test_std_dev() { let(p,i)=tri(); let s=compute_edge_length_ext_stats(&p,&i); assert!(s.std_dev>=0.0); }
    #[test] fn test_histogram() { let(p,i)=tri(); let h=edge_length_histogram(&p,&i,5); assert_eq!(h.len(),5); }
    #[test] fn test_to_json() { let(p,i)=tri(); let s=compute_edge_length_ext_stats(&p,&i); assert!(edge_length_ext_to_json(&s).contains("median")); }
    #[test] fn test_empty() { let s=compute_edge_length_ext_stats(&[],&[]); assert_eq!(s.count,0); }
    #[test] fn test_percentile_empty() { assert!((percentile(&[],0.5)).abs()<1e-6); }
}
