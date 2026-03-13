#![allow(dead_code)]

//! Face centroid computation and spatial queries.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceCentroidMap {
    pub centroids: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn build_centroid_map(positions: &[[f32; 3]], indices: &[u32]) -> FaceCentroidMap {
    let mut centroids = Vec::new();
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            let a = positions[tri[0] as usize];
            let b = positions[tri[1] as usize];
            let c = positions[tri[2] as usize];
            centroids.push([(a[0]+b[0]+c[0])/3.0,(a[1]+b[1]+c[1])/3.0,(a[2]+b[2]+c[2])/3.0]);
        }
    }
    FaceCentroidMap { centroids }
}

#[allow(dead_code)]
pub fn centroid_at_face(map: &FaceCentroidMap, face_idx: usize) -> Option<[f32; 3]> {
    map.centroids.get(face_idx).copied()
}

#[allow(dead_code)]
pub fn closest_face_to_point(map: &FaceCentroidMap, point: [f32; 3]) -> Option<usize> {
    map.centroids.iter().enumerate().min_by(|(_, a), (_, b)| {
        let da = (a[0]-point[0]).powi(2)+(a[1]-point[1]).powi(2)+(a[2]-point[2]).powi(2);
        let db = (b[0]-point[0]).powi(2)+(b[1]-point[1]).powi(2)+(b[2]-point[2]).powi(2);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    }).map(|(i, _)| i)
}

#[allow(dead_code)]
pub fn centroid_count_fcm(map: &FaceCentroidMap) -> usize {
    map.centroids.len()
}

#[allow(dead_code)]
pub fn centroid_bounds(map: &FaceCentroidMap) -> ([f32; 3], [f32; 3]) {
    if map.centroids.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut lo = map.centroids[0];
    let mut hi = map.centroids[0];
    for c in &map.centroids {
        for i in 0..3 {
            if c[i] < lo[i] { lo[i] = c[i]; }
            if c[i] > hi[i] { hi[i] = c[i]; }
        }
    }
    (lo, hi)
}

#[allow(dead_code)]
pub fn centroid_mean(map: &FaceCentroidMap) -> [f32; 3] {
    if map.centroids.is_empty() { return [0.0; 3]; }
    let mut sum = [0.0f32; 3];
    for c in &map.centroids {
        sum[0] += c[0]; sum[1] += c[1]; sum[2] += c[2];
    }
    let n = map.centroids.len() as f32;
    [sum[0]/n, sum[1]/n, sum[2]/n]
}

#[allow(dead_code)]
pub fn centroid_to_json(map: &FaceCentroidMap) -> String {
    let mean = centroid_mean(map);
    format!("{{\"count\":{},\"mean\":[{:.4},{:.4},{:.4}]}}", map.centroids.len(), mean[0], mean[1], mean[2])
}

#[allow(dead_code)]
pub fn centroid_clear_fcm(map: &mut FaceCentroidMap) {
    map.centroids.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> (Vec<[f32; 3]>, Vec<u32>) {
        (vec![[0.0,0.0,0.0],[3.0,0.0,0.0],[0.0,3.0,0.0]], vec![0,1,2])
    }

    #[test]
    fn test_build() { let (p,i)=data(); let m=build_centroid_map(&p,&i); assert_eq!(m.centroids.len(),1); }
    #[test]
    fn test_centroid_at() { let (p,i)=data(); let m=build_centroid_map(&p,&i); let c=centroid_at_face(&m,0).expect("should succeed"); assert!((c[0]-1.0).abs()<1e-6); }
    #[test]
    fn test_centroid_at_none() { let m=FaceCentroidMap{centroids:vec![]}; assert!(centroid_at_face(&m,0).is_none()); }
    #[test]
    fn test_closest() { let (p,i)=data(); let m=build_centroid_map(&p,&i); assert_eq!(closest_face_to_point(&m,[1.0,1.0,0.0]),Some(0)); }
    #[test]
    fn test_count() { let (p,i)=data(); let m=build_centroid_map(&p,&i); assert_eq!(centroid_count_fcm(&m),1); }
    #[test]
    fn test_bounds() { let (p,i)=data(); let m=build_centroid_map(&p,&i); let (lo,hi)=centroid_bounds(&m); assert!((lo[0]-hi[0]).abs()<1e-6); }
    #[test]
    fn test_mean() { let (p,i)=data(); let m=build_centroid_map(&p,&i); let mean=centroid_mean(&m); assert!((mean[0]-1.0).abs()<1e-6); }
    #[test]
    fn test_to_json() { let (p,i)=data(); let m=build_centroid_map(&p,&i); assert!(centroid_to_json(&m).contains("\"count\":1")); }
    #[test]
    fn test_clear() { let (p,i)=data(); let mut m=build_centroid_map(&p,&i); centroid_clear_fcm(&mut m); assert!(m.centroids.is_empty()); }
    #[test]
    fn test_empty_bounds() { let m=FaceCentroidMap{centroids:vec![]}; let (lo,_)=centroid_bounds(&m); assert!((lo[0]).abs()<1e-6); }
}
