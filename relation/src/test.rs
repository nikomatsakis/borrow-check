#![cfg(test)]

use rand::{self, SeedableRng};
use rand::distributions::range::Range;
use rand::distributions::IndependentSample;

use crate::vec_family::{StdVec, VecFamily};
use crate::Relation;

type StdVecRelation = Relation<StdVec<usize>>;

fn test(relation: &Relation<impl VecFamily>, expected_lines: &[&str]) {
    let actual_lines = relation.dump_and_assert();

    for (expected_line, actual_line) in expected_lines.iter().zip(&actual_lines) {
        assert_eq!(
            expected_line, actual_line,
            "expected: {:#?}\nactual:\n{:#?}\n",
            expected_lines, actual_lines,
        );
    }

    assert_eq!(
        expected_lines.len(),
        actual_lines.len(),
        "expected: {:#?}\nactual:\n{:#?}\n",
        expected_lines,
        actual_lines,
    );
}

#[test]
fn add() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);

    test(&r, &["N(0) --E(0)--> N(1)", "N(1) --E(1)--> N(2)"]);
}

#[test]
fn add_remove_1() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.remove_edges(1);

    test(&r, &["N(0) --E(0)--> N(2)", "free edge E(1)"]);
}

#[test]
fn add_remove_0() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.remove_edges(0);

    test(&r, &["N(1) --E(1)--> N(2)", "free edge E(0)"]);
}

#[test]
fn add_remove_2() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.remove_edges(2);

    test(&r, &["N(0) --E(0)--> N(1)", "free edge E(1)"]);
}

#[test]
fn add_cycle() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 0);

    test(&r, &["N(0) --E(0)--> N(1)",
               "N(1) --E(1)--> N(2)",
               "N(2) --E(2)--> N(0)",
              ]);
}

#[test]
fn remove_all() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.remove_edges(1);

    test(&r, &["N(0) --E(0)--> N(2)", "free edge E(1)"]);
    r.remove_edges(2);
    test(&r, &["free edge E(0)", "free edge E(1)"]);
}

#[test]
fn add_remove_cycle() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 0);
    r.remove_edges(1);

    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(0)",
               "free edge E(1)",
              ]);
}

#[test]
fn remove_all_cycle() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 0);
    r.remove_edges(1);

    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(0)",
               "free edge E(1)",
              ]);

    r.remove_edges(0);
    test(&r, &["N(2) --E(2)--> N(2)",
               "free edge E(0)",
               "free edge E(1)",
              ]);
}

// This test has a start graph
//
// 0 --> 2
// 1 --> 2
// 2 --> 3
//
// And wants an end graph
// 0 --> 3
// 1 --> 3
#[test]
fn remove_three_incoming_one_outgoing() {
    let mut r = StdVecRelation::new(5);

    r.add_edge(0, 2);
    r.add_edge(1, 2);
    r.add_edge(4, 2);
    r.add_edge(2, 3);

    r.remove_edges(2);
    test(&r, &["N(0) --E(0)--> N(3)",
               "N(1) --E(1)--> N(3)",
               "N(4) --E(2)--> N(3)",
               "free edge E(3)",
              ]);
}

// Start
// 0 --> 2
// 1 --> 2
// 2 --> 3
// 4 --> 2
// 5 --> 3
//
// End graph
// 0 --> 3
// 1 --> 3
// 4 --> 3
// 5 --> 3
#[test]
fn remove_three_incoming_one_outgoing_2() {
    let mut r = StdVecRelation::new(6);

    r.add_edge(0, 2);
    r.add_edge(1, 2);
    r.add_edge(4, 2);
    r.add_edge(2, 3);
    r.add_edge(5, 3);

    r.remove_edges(2);
    test(&r, &["N(0) --E(0)--> N(3)",
               "N(1) --E(1)--> N(3)",
               "N(4) --E(2)--> N(3)",
               "N(5) --E(4)--> N(3)",
               "free edge E(3)",
              ]);
}

// This test has a start graph
//
// 0 --> 1
// 1 --> 2
// 1 --> 3
//
// And wants an end graph
// 0 --> 2
// 0 --> 3
#[test]
fn remove_one_incoming_two_outgoing() {
    let mut r = StdVecRelation::new(4);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(1, 3);

    test(&r, &["N(0) --E(0)--> N(1)",
               "N(1) --E(2)--> N(3)",
               "N(1) --E(1)--> N(2)",
              ]);

    r.remove_edges(1);
    test(&r, &["N(0) --E(1)--> N(2)",
               "N(0) --E(2)--> N(3)",
               "free edge E(0)",
              ]);
}

// Graph From:
// 0 --> 1
// 1 --> 2
// 3 --> 2
//
// Graph To:
// 0 --> 2
// 3 --> 2
//
#[test]
fn add_remove_complex_1() {
    let mut r = StdVecRelation::new(4);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(3, 2);

    r.remove_edges(1);

    test(&r, &["N(0) --E(0)--> N(2)",
               "N(3) --E(2)--> N(2)",
               "free edge E(1)",
              ]);
}

#[test]
fn long_remove_cycle() {
    let mut r = StdVecRelation::new(6);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 3);
    r.add_edge(3, 4);
    r.add_edge(4, 0);

    test(&r, &["N(0) --E(0)--> N(1)",
               "N(1) --E(1)--> N(2)",
               "N(2) --E(2)--> N(3)",
               "N(3) --E(3)--> N(4)",
               "N(4) --E(4)--> N(0)",
              ]);

    r.remove_edges(1);
    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(3)",
               "N(3) --E(3)--> N(4)",
               "N(4) --E(4)--> N(0)",
               "free edge E(1)",
              ]);

    r.remove_edges(3);
    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(4)",
               "N(4) --E(4)--> N(0)",
               "free edge E(3)",
               "free edge E(1)",
              ]);

    r.remove_edges(0);
    test(&r, &["N(2) --E(2)--> N(4)",
               "N(4) --E(4)--> N(2)",
               "free edge E(0)",
               "free edge E(3)",
               "free edge E(1)",
              ]);
}

#[test]
fn multi_in_multi_out() {
    let mut r = StdVecRelation::new(5);

    r.add_edge(0, 2);
    r.add_edge(1, 2);
    r.add_edge(2, 3);
    r.add_edge(2, 4);
    test(&r, &["N(0) --E(0)--> N(2)",
               "N(1) --E(1)--> N(2)",
               "N(2) --E(3)--> N(4)",
               "N(2) --E(2)--> N(3)",
              ]);

    r.remove_edges(2);
    test(&r, &["N(0) --E(3)--> N(3)",
               "N(0) --E(1)--> N(4)",
               "N(1) --E(2)--> N(3)",
               "N(1) --E(0)--> N(4)",
              ]);
}

#[test]
fn scratch_random() {
    let mut r = StdVecRelation::new(1000);
    let range = Range::new(0, 1000);

    let mut rng = rand::StdRng::from_seed(&[1,2,3,4]);
    let rng = &mut rng;

    for _ in 0..300 {
        let (mut src, mut dst) = (range.ind_sample(rng), range.ind_sample(rng));
        while src == dst {
            src = range.ind_sample(rng);
            dst = range.ind_sample(rng);
        }
        println!("r.add_edge({}, {});", src, dst);
        r.add_edge(src, dst);
    }

    for i in 0..1000 {
        println!("r.remove_edges({});", i);
        r.remove_edges(i);
    }
}

#[test]
fn scratch_explicit() {
    // This case started by using the operations from above, and then
    // deleting rows that still made it fail
    let mut r = StdVecRelation::new(1000);
//    r.add_edge(426, 669);
//    r.add_edge(847, 594);
//    r.add_edge(452, 54);
//    r.add_edge(585, 178);
//    r.add_edge(240, 92);
//    r.add_edge(557, 356);
//    r.add_edge(432, 258);
//    r.add_edge(18, 619);
    r.add_edge(449, 10);
//    r.add_edge(662, 24);
    //r.add_edge(557, 946);
    //r.add_edge(748, 663);
//    r.add_edge(570, 822);
//    r.add_edge(464, 892);
//    r.add_edge(932, 16);
//    r.add_edge(747, 14);
//    r.add_edge(544, 429);
//    r.add_edge(434, 352);
//    r.add_edge(811, 363);
//    r.add_edge(421, 671);
//    r.add_edge(48, 814);
//    r.add_edge(346, 973);
//    r.add_edge(249, 322);
//    r.add_edge(167, 56);
//    r.add_edge(245, 570);
//    r.add_edge(901, 498);
    //r.add_edge(832, 388);
    //r.add_edge(16, 551);
    //r.add_edge(927, 173);
    //r.add_edge(787, 860);
    //r.add_edge(185, 632);
    //r.add_edge(957, 228);
    //r.add_edge(689, 819);
    //r.add_edge(337, 15);
    //r.add_edge(180, 670);
    //r.add_edge(608, 242);
    //r.add_edge(514, 481);
    //r.add_edge(344, 459);
    //r.add_edge(58, 634);
    //r.add_edge(154, 527);
    //r.add_edge(266, 405);
    //r.add_edge(759, 585);
    //r.add_edge(551, 304);
    //r.add_edge(313, 759);
    //r.add_edge(17, 964);
    //r.add_edge(236, 298);
    //r.add_edge(792, 762);
    //r.add_edge(266, 576);
    //r.add_edge(199, 785);
    //r.add_edge(787, 209);
    //r.add_edge(450, 990);
    //r.add_edge(702, 36);
    //r.add_edge(38, 186);
    //r.add_edge(874, 759);
    //r.add_edge(813, 467);
    //r.add_edge(824, 420);
    //r.add_edge(969, 249);
    //r.add_edge(295, 74);
    //r.add_edge(429, 642);
    //r.add_edge(43, 888);
    //r.add_edge(373, 135);
    //r.add_edge(452, 802);
    //r.add_edge(543, 369);
    //r.add_edge(341, 126);
    //r.add_edge(389, 531);
    //r.add_edge(25, 182);
    //r.add_edge(698, 922);
    //r.add_edge(736, 182);
    //r.add_edge(508, 628);
    //r.add_edge(326, 956);
    //r.add_edge(578, 992);
    //r.add_edge(476, 706);
    //r.add_edge(589, 54);
    //r.add_edge(907, 360);
    //r.add_edge(665, 344);
    //r.add_edge(801, 422);
    //r.add_edge(482, 359);
    //r.add_edge(897, 333);
    //r.add_edge(306, 335);
    //r.add_edge(692, 326);
    //r.add_edge(813, 677);
    //r.add_edge(589, 712);
    //r.add_edge(953, 184);
    //r.add_edge(181, 742);
    //r.add_edge(176, 683);
    //r.add_edge(427, 233);
    //r.add_edge(887, 485);
    //r.add_edge(100, 130);
    //r.add_edge(73, 390);
    //r.add_edge(46, 64);
    //r.add_edge(864, 64);
    //r.add_edge(542, 800);
    //r.add_edge(147, 20);
    //r.add_edge(90, 929);
    //r.add_edge(717, 371);
    //r.add_edge(445, 639);
    //r.add_edge(585, 576);
    //r.add_edge(148, 978);
    //r.add_edge(222, 292);
    //r.add_edge(185, 844);
    //r.add_edge(415, 726);
    //r.add_edge(481, 150);
    //r.add_edge(249, 311);
    //r.add_edge(163, 316);
    //r.add_edge(540, 698);
    //r.add_edge(715, 639);
    //r.add_edge(363, 11);
    //r.add_edge(28, 786);
    //r.add_edge(955, 872);
    //r.add_edge(599, 162);
    //r.add_edge(79, 871);
    //r.add_edge(848, 692);
    //r.add_edge(516, 686);
    //r.add_edge(261, 831);
    //r.add_edge(328, 827);
    //r.add_edge(203, 921);
    //r.add_edge(151, 189);
    //r.add_edge(358, 718);
    //r.add_edge(113, 792);
    //r.add_edge(764, 546);
    //r.add_edge(144, 193);
    //r.add_edge(257, 802);
    //r.add_edge(257, 960);
    //r.add_edge(801, 980);
    //r.add_edge(286, 751);
    //r.add_edge(565, 756);
    //r.add_edge(155, 826);
    //r.add_edge(712, 990);
    //r.add_edge(839, 836);
    //r.add_edge(953, 926);
    //r.add_edge(171, 318);
//    r.add_edge(230, 644);
//    r.add_edge(277, 819);
//    r.add_edge(858, 443);
//    r.add_edge(662, 869);
//    r.add_edge(686, 149);
//    r.add_edge(788, 959);
//    r.add_edge(524, 327);
//    r.add_edge(319, 17);
//    r.add_edge(642, 427);
//    r.add_edge(276, 936);
//    r.add_edge(650, 299);
//    r.add_edge(463, 415);
//    r.add_edge(36, 842);
//    r.add_edge(913, 55);
//    r.add_edge(627, 917);
//    r.add_edge(145, 985);
//    r.add_edge(461, 312);
//    r.add_edge(883, 630);
//    r.add_edge(482, 735);
//    r.add_edge(553, 40);
//    r.add_edge(386, 373);
//    r.add_edge(447, 71);
//    r.add_edge(926, 896);
//    r.add_edge(287, 924);
//    r.add_edge(412, 179);
//    r.add_edge(398, 686);
//    r.add_edge(545, 146);
//    r.add_edge(191, 194);
//    r.add_edge(964, 176);
    r.add_edge(617, 456);
    r.add_edge(711, 196); // NB
    r.add_edge(417, 720);
    r.add_edge(401, 315);
    r.add_edge(595, 756);
//    r.add_edge(116, 907);
//    r.add_edge(242, 648);
//    r.add_edge(643, 488);
//    r.add_edge(611, 880);
//    r.add_edge(668, 927);
//    r.add_edge(129, 562);
//    r.add_edge(204, 785);
//    r.add_edge(880, 723);
//    r.add_edge(651, 149);
//    r.add_edge(852, 431);
//    r.add_edge(489, 537);
//    r.add_edge(864, 741);
//    r.add_edge(974, 41);
//    r.add_edge(306, 856);
//    r.add_edge(525, 760);
//    r.add_edge(210, 833);
//    r.add_edge(846, 878);
//    r.add_edge(417, 739);
//    r.add_edge(417, 870);
//    r.add_edge(877, 832);
//    r.add_edge(324, 640);
//    r.add_edge(806, 168);
//    r.add_edge(79, 715);
//    r.add_edge(550, 589);
//    r.add_edge(198, 709);
//    r.add_edge(131, 137);
//    r.add_edge(647, 296);
//    r.add_edge(982, 762);
//    r.add_edge(256, 891);
//    r.add_edge(813, 61);
//    r.add_edge(618, 972);
//    r.add_edge(646, 116);
//    r.add_edge(188, 674);
//    r.add_edge(656, 177);
//    r.add_edge(430, 604);
    r.add_edge(736, 210);
    r.add_edge(746, 326);
    r.add_edge(680, 903);
    r.add_edge(196, 873); // NB
    //r.add_edge(760, 940);
    //r.add_edge(547, 721);
    //r.add_edge(915, 895);
    //r.add_edge(876, 206);
    //r.add_edge(267, 289);
    //r.add_edge(922, 739);
    //r.add_edge(723, 717);
    //r.add_edge(683, 677);
    //r.add_edge(346, 508);
    //r.add_edge(571, 701);
    //r.add_edge(927, 999);
    //r.add_edge(54, 99);
    //r.add_edge(522, 463);
    //r.add_edge(126, 542);
    //r.add_edge(863, 247);
    //r.add_edge(430, 694);
    //r.add_edge(99, 440);
    //r.add_edge(831, 893);
    //r.add_edge(328, 279);
    //r.add_edge(660, 984);
    //r.add_edge(594, 442);
    //r.add_edge(308, 745);
    //r.add_edge(778, 327);
    //r.add_edge(905, 949);
    //r.add_edge(422, 862);
    //r.add_edge(995, 83);
    //r.add_edge(864, 275);
    //r.add_edge(931, 358);
    //r.add_edge(424, 294);
    //r.add_edge(537, 546);
    //r.add_edge(985, 298);
    //r.add_edge(878, 666);
    //r.add_edge(688, 952);
    //r.add_edge(198, 919);
    //r.add_edge(867, 740);
    //r.add_edge(212, 900);
    //r.add_edge(637, 444);
    //r.add_edge(610, 142);
    //r.add_edge(534, 458);
    //r.add_edge(550, 504);
    //r.add_edge(410, 653);
    //r.add_edge(93, 373);
    //r.add_edge(411, 970);
    //r.add_edge(305, 96);
    //r.add_edge(83, 949);
    //r.add_edge(406, 993);
    //r.add_edge(662, 43);
    //r.add_edge(810, 484);
    //r.add_edge(541, 660);
    //r.add_edge(361, 365);
    //r.add_edge(652, 312);
    //r.add_edge(785, 320);
    //r.add_edge(72, 704);
    //r.add_edge(775, 54);
    //r.add_edge(506, 393);
    //r.add_edge(960, 930);
    //r.add_edge(887, 262);
    //r.add_edge(804, 614);
    //r.add_edge(51, 803);
    //r.add_edge(573, 55);
    //r.add_edge(303, 99);
    r.add_edge(355, 343);
    r.add_edge(750, 666);
    r.add_edge(10, 873);
    r.add_edge(427, 981);
    r.add_edge(35, 359);
    r.add_edge(391, 215);
    //r.add_edge(821, 426);
    //r.add_edge(476, 646);
    //r.add_edge(312, 425);
    //r.add_edge(325, 351);
    //r.add_edge(995, 628);
    //r.add_edge(24, 581);
    //r.add_edge(691, 743);
    //r.add_edge(166, 174);
    //r.add_edge(360, 595);
    //r.add_edge(93, 308);
    //r.add_edge(697, 777);
    //r.add_edge(110, 804);
    //r.add_edge(672, 21);
    //r.add_edge(79, 381);
    //r.add_edge(544, 636);
    //r.add_edge(548, 141);
    //r.add_edge(677, 341);
    //r.add_edge(860, 170);
    //r.add_edge(22, 669);
    //r.add_edge(127, 436);
    //r.add_edge(328, 223);
    //r.add_edge(92, 8);
    //r.add_edge(705, 310);
    //r.add_edge(48, 995);
    //r.add_edge(582, 783);
    //r.add_edge(732, 48);
    //r.add_edge(162, 443);
    //r.add_edge(389, 377);
    //r.add_edge(787, 891);
    r.remove_edges(0);
    r.remove_edges(1);
    r.remove_edges(2);
    r.remove_edges(3);
    r.remove_edges(4);
    r.remove_edges(5);
    r.remove_edges(6);
    r.remove_edges(7);
    r.remove_edges(8);
    r.remove_edges(9);
    r.remove_edges(10);
    r.remove_edges(11);
    r.remove_edges(12);
    r.remove_edges(13);
    r.remove_edges(14);
    r.remove_edges(15);
    r.remove_edges(16);
    r.remove_edges(17);
    r.remove_edges(18);
    r.remove_edges(19);
    r.remove_edges(20);
    r.remove_edges(21);
    r.remove_edges(22);
    r.remove_edges(23);
    r.remove_edges(24);
    r.remove_edges(25);
    r.remove_edges(26);
    r.remove_edges(27);
    r.remove_edges(28);
    r.remove_edges(29);
    r.remove_edges(30);
    r.remove_edges(31);
    r.remove_edges(32);
    r.remove_edges(33);
    r.remove_edges(34);
    r.remove_edges(35);
    r.remove_edges(36);
    r.remove_edges(37);
    r.remove_edges(38);
    r.remove_edges(39);
    r.remove_edges(40);
    r.remove_edges(41);
    r.remove_edges(42);
    r.remove_edges(43);
    r.remove_edges(44);
    r.remove_edges(45);
    r.remove_edges(46);
    r.remove_edges(47);
    r.remove_edges(48);
    r.remove_edges(49);
    r.remove_edges(50);
    r.remove_edges(51);
    r.remove_edges(52);
    r.remove_edges(53);
    r.remove_edges(54);
    r.remove_edges(55);
    r.remove_edges(56);
    r.remove_edges(57);
    r.remove_edges(58);
    r.remove_edges(59);
    r.remove_edges(60);
    r.remove_edges(61);
    r.remove_edges(62);
    r.remove_edges(63);
    r.remove_edges(64);
    r.remove_edges(65);
    r.remove_edges(66);
    r.remove_edges(67);
    r.remove_edges(68);
    r.remove_edges(69);
    r.remove_edges(70);
    r.remove_edges(71);
    r.remove_edges(72);
    r.remove_edges(73);
    r.remove_edges(74);
    r.remove_edges(75);
    r.remove_edges(76);
    r.remove_edges(77);
    r.remove_edges(78);
    r.remove_edges(79);
    r.remove_edges(80);
    r.remove_edges(81);
    r.remove_edges(82);
    r.remove_edges(83);
    r.remove_edges(84);
    r.remove_edges(85);
    r.remove_edges(86);
    r.remove_edges(87);
    r.remove_edges(88);
    r.remove_edges(89);
    r.remove_edges(90);
    r.remove_edges(91);
    r.remove_edges(92);
    r.remove_edges(93);
    r.remove_edges(94);
    r.remove_edges(95);
    r.remove_edges(96);
    r.remove_edges(97);
    r.remove_edges(98);
    r.remove_edges(99);
    r.remove_edges(100);
    r.remove_edges(101);
    r.remove_edges(102);
    r.remove_edges(103);
    r.remove_edges(104);
    r.remove_edges(105);
    r.remove_edges(106);
    r.remove_edges(107);
    r.remove_edges(108);
    r.remove_edges(109);
    r.remove_edges(110);
    r.remove_edges(111);
    r.remove_edges(112);
    r.remove_edges(113);
    r.remove_edges(114);
    r.remove_edges(115);
    r.remove_edges(116);
    r.remove_edges(117);
    r.remove_edges(118);
    r.remove_edges(119);
    r.remove_edges(120);
    r.remove_edges(121);
    r.remove_edges(122);
    r.remove_edges(123);
    r.remove_edges(124);
    r.remove_edges(125);
    r.remove_edges(126);
    r.remove_edges(127);
    r.remove_edges(128);
    r.remove_edges(129);
    r.remove_edges(130);
    r.remove_edges(131);
    r.remove_edges(132);
    r.remove_edges(133);
    r.remove_edges(134);
    r.remove_edges(135);
    r.remove_edges(136);
    r.remove_edges(137);
    r.remove_edges(138);
    r.remove_edges(139);
    r.remove_edges(140);
    r.remove_edges(141);
    r.remove_edges(142);
    r.remove_edges(143);
    r.remove_edges(144);
    r.remove_edges(145);
    r.remove_edges(146);
    r.remove_edges(147);
    r.remove_edges(148);
    r.remove_edges(149);
    r.remove_edges(150);
    r.remove_edges(151);
    r.remove_edges(152);
    r.remove_edges(153);
    r.remove_edges(154);
    r.remove_edges(155);
    r.remove_edges(156);
    r.remove_edges(157);
    r.remove_edges(158);
    r.remove_edges(159);
    r.remove_edges(160);
    r.remove_edges(161);
    r.remove_edges(162);
    r.remove_edges(163);
    r.remove_edges(164);
    r.remove_edges(165);
    r.remove_edges(166);
    r.remove_edges(167);
    r.remove_edges(168);
    r.remove_edges(169);
    r.remove_edges(170);
    r.remove_edges(171);
    r.remove_edges(172);
    r.remove_edges(173);
    r.remove_edges(174);
    r.remove_edges(175);
    r.remove_edges(176);
    r.remove_edges(177);
    r.remove_edges(178);
    r.remove_edges(179);
    r.remove_edges(180);
    r.remove_edges(181);
    r.remove_edges(182);
    r.remove_edges(183);
    r.remove_edges(184);
    r.remove_edges(185);
    r.remove_edges(186);
    r.remove_edges(187);
    r.remove_edges(188);
    r.remove_edges(189);
    r.remove_edges(190);
    r.remove_edges(191);
    r.remove_edges(192);
    r.remove_edges(193);
    r.remove_edges(194);
    r.remove_edges(195);
    r.remove_edges(196);
}
