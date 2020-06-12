
use coitrees::{COITree, IntervalNode, SortedQuerent};

extern crate rand;
use rand::{Rng, thread_rng};


// True iff the two intervals overlap.
#[inline(always)]
fn overlaps(first_a: i32, last_a: i32, first_b: i32, last_b: i32) -> bool {
    return first_a <= last_b && last_a >= first_b;
}


// Find overlapping intervals by simply checking every single one.
// We test against this algorithm which we assume to be correct.
fn brute_force_query<T, F>(
        intervals: &[IntervalNode<T>], query_first: i32, query_last: i32, mut visit: F)
            where T: Copy, F: FnMut(&IntervalNode<T>) {
    for interval in intervals {
        if overlaps(interval.first, interval.last, query_first, query_last) {
            visit(interval);
        }
    }
}


// Brute coverage calculation. `intervals` must be sorted.
fn brute_force_coverage<T>(
        intervals: &[IntervalNode<T>], query_first: i32, query_last: i32) -> f64
            where T: Copy {
    let mut last_cov = query_first - 1;
    let mut uncov_len = 0;
    for interval in intervals {
        if overlaps(interval.first, interval.last, query_first, query_last) {
            if interval.first > last_cov {
                uncov_len += interval.first - (last_cov + 1);
            }
            last_cov = last_cov.max(interval.last);
        }
    }
    if last_cov < query_last {
        uncov_len += query_last - last_cov;
    }

    return 1.0 - (uncov_len as f64) / ((query_last - query_first + 1) as f64);
}


fn print_brute_force_coverage<T>(
        intervals: &[IntervalNode<T>], query_first: i32, query_last: i32) -> f64
            where T: Copy {
    let mut last_cov = query_first - 1;
    let mut uncov_len = 0;
    eprintln!("first, last = {}, {}", query_first, query_last);
    for interval in intervals {
        if overlaps(interval.first, interval.last, query_first, query_last) {
            eprintln!("interval.first, interval.last = {}, {}", interval.first, interval.last);
            if interval.first > last_cov {
                uncov_len += interval.first - (last_cov + 1);
            }
            last_cov = last_cov.max(interval.last);
        }
    }
    if last_cov < query_last {
        uncov_len += query_last - last_cov;
    }

    return 1.0 - (uncov_len as f64) / ((query_last - query_first + 1) as f64);
}


// Run queries against both a COITree and by brute force and check that
// they get the same results.
fn check_queries(a: &COITree<u32>, b: &[IntervalNode<u32>], queries: &mut [(i32, i32)]) {
    let mut a_hits: Vec<u32> = Vec::new();
    let mut b_hits: Vec<u32> = Vec::new();

    for (query_first, query_last) in queries {
        a_hits.clear();
        b_hits.clear();

        a.query(*query_first, *query_last, |node| {
            a_hits.push(node.metadata)
        });

        brute_force_query(b, *query_first, *query_last, |node| {
            b_hits.push(node.metadata)
        });

        a_hits.sort();
        b_hits.sort();

        assert_eq!(a_hits, b_hits);
    }
}


fn check_coverage(a: &COITree<u32>, b: &[IntervalNode<u32>], queries: &mut [(i32, i32)]) {
    for (query_first, query_last) in queries {
        let a_cover = a.coverage(*query_first, *query_last);
        let b_cover = brute_force_coverage(b, *query_first, *query_last);

        if (a_cover - b_cover).abs() >= 1e-8 {
            eprintln!("{} {}", a_cover, b_cover);
            print_brute_force_coverage(b, *query_first, *query_last);
        }
        assert!((a_cover - b_cover).abs() < 1e-8);
    }
}


fn check_count_queries(a: &COITree<u32>, b: &[IntervalNode<u32>], queries: &mut [(i32, i32)]) {
    for (query_first, query_last) in queries {
        let a_cnt = a.query_count(*query_first, *query_last);

        let mut b_cnt = 0;
        brute_force_query(b, *query_first, *query_last, |_| {
            b_cnt += 1;
        });

        assert_eq!(a_cnt, b_cnt);
    }
}


// check SortedQuerent queries against brute force
fn check_sorted_querent_queries(
        a: &COITree<u32>, b: &[IntervalNode<u32>], queries: &mut [(i32, i32)]) {
    let mut a_hits: Vec<u32> = Vec::new();
    let mut b_hits: Vec<u32> = Vec::new();

    let mut qa = SortedQuerent::new(a);

    queries.sort();

    for (query_first, query_last) in queries {
        a_hits.clear();
        b_hits.clear();

        qa.query(*query_first, *query_last, |node| {
            a_hits.push(node.metadata)
        });

        brute_force_query(b, *query_first, *query_last, |node| {
            b_hits.push(node.metadata)
        });

        a_hits.sort();
        b_hits.sort();

        assert_eq!(a_hits, b_hits);
    }
}


// check that SortedQuerent still works even when queries are unsorted
fn check_sorted_querent_unsorted_queries(
        a: &COITree<u32>, b: &[IntervalNode<u32>], queries: &mut [(i32, i32)]) {
    let mut a_hits: Vec<u32> = Vec::new();
    let mut b_hits: Vec<u32> = Vec::new();

    let mut qa = SortedQuerent::new(a);

    for (query_first, query_last) in queries {
        a_hits.clear();
        b_hits.clear();

        qa.query(*query_first, *query_last, |node| {
            a_hits.push(node.metadata)
        });

        brute_force_query(b, *query_first, *query_last, |node| {
            b_hits.push(node.metadata)
        });

        a_hits.sort();
        b_hits.sort();

        assert_eq!(a_hits, b_hits);
    }
}


fn random_interval(min_first: i32, max_last: i32, min_len: i32, max_len: i32) -> (i32, i32) {
    let mut rng = thread_rng();
    let len = rng.gen_range(min_len, max_len+1);
    let start = rng.gen_range(min_first, max_last - len + 1);
    return (start, start+len-1)
}


fn check_random_queries<F>(
        n: usize, num_queries: usize, max_last: i32,
        min_len: i32, max_len: i32,
        query_min_len: i32, query_max_len: i32,
        check: F)
        where F: Fn(&COITree<u32>, &[IntervalNode<u32>], &mut [(i32, i32)]) {
    let min_first = 0;

    let mut b: Vec<IntervalNode<u32>> = Vec::with_capacity(n);
    for i in 0..n {
        let (first, last) = random_interval(min_first, max_last, min_len, max_len);
        b.push(IntervalNode::new(first, last, i as u32));
    }
    b.sort_unstable_by_key(|node| node.first);

    let a = COITree::new(b.clone());

    let mut queries: Vec<(i32, i32)> = Vec::with_capacity(num_queries);
    for _ in 0..num_queries {
        queries.push(random_interval(min_first, max_last, query_min_len, query_max_len));
    }

    check(&a, &b, &mut queries);
}


fn check_random_queries_default<F>(n: usize, num_queries: usize, check: F)
        where F: Fn(&COITree<u32>, &[IntervalNode<u32>], &mut [(i32, i32)]) {

    let max_last = 1000000;
    let min_len = 20;
    let max_len = 2000;

    check_random_queries(
        n, num_queries, max_last, min_len, max_len, min_len, max_len, check);
}


const CHECKS: [fn(&COITree<u32>, &[IntervalNode<u32>], &mut [(i32, i32)]); 4] =
    [
        check_queries,
        check_count_queries,
        check_sorted_querent_queries,
        check_sorted_querent_unsorted_queries
    ];

#[test]
fn query_empty_tree() {
    for check in &CHECKS {
        check_random_queries_default(0, 1000, check);
    }
    check_random_queries_default(0, 1000, check_coverage);
}

#[test]
fn query_small_trees() {
    for n in 1..16 {
        for check in &CHECKS {
            check_random_queries_default(n, 1000, check);
        }
        check_random_queries_default(n, 1000, check_coverage);
    }
}

#[test]
fn query_medium_tree() {
    for check in &CHECKS {
        check_random_queries_default(10000, 1000, check);
    }
    check_random_queries_default(10000, 1000, check_coverage);
}


#[test]
fn query_singeton_intervals() {
    for check in &CHECKS {
        check_random_queries(10000, 1000, 1000, 1, 1, 1,    1, check);
        check_random_queries(10000, 1000, 1000, 1, 1, 10, 100, check);
    }
    check_random_queries(10000, 1000, 1000, 1, 1, 1,    1, check_coverage);
    check_random_queries(10000, 1000, 1000, 1, 1, 10, 100, check_coverage);
}


#[test]
fn query_empty_intervals() {
    for check in &CHECKS {
        check_random_queries(10000, 1000, 1000, 0, 0, 0,    0, check);
        check_random_queries(10000, 1000, 1000, 0, 0, 10, 100, check);
    }
}
