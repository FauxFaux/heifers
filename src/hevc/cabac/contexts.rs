const NUMBER_OF_SLICE_TYPES: usize = 3;
const CONTEXT_NUMBER_OF_TYPES: usize = 4;
const MAX_NUM_CHANNEL_TYPE: usize = 2;

/// maximum number of supported contexts
const MAX_NUM_CTX_MOD: usize = 512;

/// number of context models for split flag
const NUM_SPLIT_FLAG_CTX: usize = 3;
/// number of context models for skip flag
const NUM_SKIP_FLAG_CTX: usize = 3;

/// number of context models for merge flag of merge extended
const NUM_MERGE_FLAG_EXT_CTX: usize = 1;
/// number of context models for merge index of merge extended
const NUM_MERGE_IDX_EXT_CTX: usize = 1;

/// number of context models for partition size
const NUM_PART_SIZE_CTX: usize = 4;
/// number of context models for prediction mode
const NUM_PRED_MODE_CTX: usize = 1;

/// number of context models for intra prediction
const NUM_INTRA_PREDICT_CTX: usize = 1;

/// number of context models for intra prediction (chroma)
const NUM_CHROMA_PRED_CTX: usize = 2;
/// number of context models for inter prediction direction
const NUM_INTER_DIR_CTX: usize = 5;
/// number of context models for motion vector difference
const NUM_MV_RES_CTX: usize = 2;
/// number of context models for chroma_qp_adjustment_flag
const NUM_CHROMA_QP_ADJ_FLAG_CTX: usize = 1;
/// number of context models for chroma_qp_adjustment_idc
const NUM_CHROMA_QP_ADJ_IDC_CTX: usize = 1;

/// number of context models for reference index
const NUM_REF_NO_CTX: usize = 2;
/// number of context models for transform subdivision flags
const NUM_TRANS_SUBDIV_FLAG_CTX: usize = 3;
/// number of context models for QT ROOT CBF
const NUM_QT_ROOT_CBF_CTX: usize = 1;
/// number of context models for dQP
const NUM_DELTA_QP_CTX: usize = 3;

/// number of context models for MULTI_LEVEL_SIGNIFICANCE
const NUM_SIG_CG_FLAG_CTX: usize = 2;
/// number of context models for the flag which specifies whether to use RDPCM on inter coded residues
const NUM_EXPLICIT_RDPCM_FLAG_CTX: usize = 1;
/// number of context models for the flag which specifies which RDPCM direction is used on inter coded residues
const NUM_EXPLICIT_RDPCM_DIR_CTX: usize = 1;

//--------------------------------------------------------------------------------------------------

// context size definitions for significance map

/// number of context models for luma sig flag
const NUM_SIG_FLAG_CTX_LUMA: usize = 28;
/// number of context models for chroma sig flag
const NUM_SIG_FLAG_CTX_CHROMA: usize = 16;

//                                                                                                       |----Luma-----|  |---Chroma----|
#[rustfmt::skip]
const significanceMapContextSetStart         : [[u8; CONTEXT_NUMBER_OF_TYPES]; MAX_NUM_CHANNEL_TYPE] = [ [0,  9, 21, 27], [0,  9, 12, 15] ];
#[rustfmt::skip]
const significanceMapContextSetSize          : [[u8; CONTEXT_NUMBER_OF_TYPES]; MAX_NUM_CHANNEL_TYPE] = [ [9, 12,  6,  1], [9,  3,  3,  1] ];
#[rustfmt::skip]
const nonDiagonalScan8x8ContextOffset        : [ u8; MAX_NUM_CHANNEL_TYPE]                           = [  6,               0              ];
#[rustfmt::skip]
const notFirstGroupNeighbourhoodContextOffset: [ u8; MAX_NUM_CHANNEL_TYPE]                           = [  3,               0              ];

/// number of context models for sig flag
const NUM_SIG_FLAG_CTX: usize = NUM_SIG_FLAG_CTX_LUMA + NUM_SIG_FLAG_CTX_CHROMA;

//--------------------------------------------------------------------------------------------------

// context size definitions for last significant coefficient position

const NUM_CTX_LAST_FLAG_SETS: usize = 2;

/// number of context models for last coefficient position
const NUM_CTX_LAST_FLAG_XY: usize = 15;

//--------------------------------------------------------------------------------------------------

// context size definitions for greater-than-one and greater-than-two maps

/// number of context models for greater than 1 flag in a set
const NUM_ONE_FLAG_CTX_PER_SET: usize = 4;
/// number of context models for greater than 2 flag in a set
const NUM_ABS_FLAG_CTX_PER_SET: usize = 1;

//------------------

/// number of context model sets for luminance
const NUM_CTX_SETS_LUMA: usize = 4;
/// number of context model sets for combined chrominance
const NUM_CTX_SETS_CHROMA: usize = 2;

/// index of first luminance context set
const FIRST_CTX_SET_LUMA: usize = 0;

//------------------
/// number of context models for greater than 1 flag of luma
const NUM_ONE_FLAG_CTX_LUMA: usize = (NUM_ONE_FLAG_CTX_PER_SET * NUM_CTX_SETS_LUMA);
/// number of context models for greater than 1 flag of chroma
const NUM_ONE_FLAG_CTX_CHROMA: usize = (NUM_ONE_FLAG_CTX_PER_SET * NUM_CTX_SETS_CHROMA);

/// number of context models for greater than 2 flag of luma
const NUM_ABS_FLAG_CTX_LUMA: usize = (NUM_ABS_FLAG_CTX_PER_SET * NUM_CTX_SETS_LUMA);
/// number of context models for greater than 2 flag of chroma
const NUM_ABS_FLAG_CTX_CHROMA: usize = (NUM_ABS_FLAG_CTX_PER_SET * NUM_CTX_SETS_CHROMA);

/// number of context models for greater than 1 flag
const NUM_ONE_FLAG_CTX: usize = (NUM_ONE_FLAG_CTX_LUMA + NUM_ONE_FLAG_CTX_CHROMA);
/// number of context models for greater than 2 flag
const NUM_ABS_FLAG_CTX: usize = (NUM_ABS_FLAG_CTX_LUMA + NUM_ABS_FLAG_CTX_CHROMA);

/// index of first chrominance context set
const FIRST_CTX_SET_CHROMA: usize = (FIRST_CTX_SET_LUMA + NUM_CTX_SETS_LUMA);

//--------------------------------------------------------------------------------------------------

// context size definitions for CBF

const NUM_QT_CBF_CTX_SETS: usize = 2;

/// number of context models for QT CBF
const NUM_QT_CBF_CTX_PER_SET: usize = 5;

/// index of first luminance CBF context
const FIRST_CBF_CTX_LUMA: usize = 0;

///< index of first chrominance CBF context
const FIRST_CBF_CTX_CHROMA: usize = (FIRST_CBF_CTX_LUMA + NUM_QT_CBF_CTX_PER_SET);

//--------------------------------------------------------------------------------------------------

/// number of context models for MVP index
const NUM_MVP_IDX_CTX: usize = 1;

/// number of context models for SAO merge flags
const NUM_SAO_MERGE_FLAG_CTX: usize = 1;
/// number of context models for SAO type index
const NUM_SAO_TYPE_IDX_CTX: usize = 1;

/// number of context models for transform skipping
const NUM_TRANSFORMSKIP_FLAG_CTX: usize = 1;

const NUM_CU_TRANSQUANT_BYPASS_FLAG_CTX: usize = 1;

const NUM_CROSS_COMPONENT_PREDICTION_CTX: usize = 10;

/// dummy initialization value for unused context models 'Context model Not Used'
const CNU: u8 = 154;

// ====================================================================================================================
// Tables
// ====================================================================================================================

// initial probability for cu_transquant_bypass flag

#[rustfmt::skip]
const INIT_CU_TRANSQUANT_BYPASS_FLAG: [[u8; NUM_CU_TRANSQUANT_BYPASS_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 154 ],
    [ 154 ],
    [ 154 ],
];

// initial probability for split flag
#[rustfmt::skip]
const INIT_SPLIT_FLAG: [[u8; NUM_SPLIT_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 107,  139,  126, ],
    [ 107,  139,  126, ],
    [ 139,  141,  157, ],
];

#[rustfmt::skip]
const INIT_SKIP_FLAG: [[u8; NUM_SKIP_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 197,  185,  201, ],
    [ 197,  185,  201, ],
    [ CNU,  CNU,  CNU, ],
];

#[rustfmt::skip]
const INIT_MERGE_FLAG_EXT: [[u8; NUM_MERGE_FLAG_EXT_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 154, ],
    [ 110, ],
    [ CNU, ],
];

#[rustfmt::skip]
const INIT_MERGE_IDX_EXT: [[u8; NUM_MERGE_IDX_EXT_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 137, ],
    [ 122, ],
    [ CNU, ],
];

#[rustfmt::skip]
const INIT_PART_SIZE: [[u8; NUM_PART_SIZE_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 154,  139,  154, 154 ],
    [ 154,  139,  154, 154 ],
    [ 184,  CNU,  CNU, CNU ],
];

#[rustfmt::skip]
const INIT_PRED_MODE: [[u8; NUM_PRED_MODE_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 134, ],
    [ 149, ],
    [ CNU, ],
];

#[rustfmt::skip]
const INIT_INTRA_PRED_MODE: [[u8; NUM_INTRA_PREDICT_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 183, ],
    [ 154, ],
    [ 184, ],
];

#[rustfmt::skip]
const INIT_CHROMA_PRED_MODE: [[u8; NUM_CHROMA_PRED_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 152,  139, ],
    [ 152,  139, ],
    [  63,  139, ],
];

#[rustfmt::skip]
const INIT_INTER_DIR: [[u8; NUM_INTER_DIR_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [  95,   79,   63,   31,  31, ],
    [  95,   79,   63,   31,  31, ],
    [ CNU,  CNU,  CNU,  CNU, CNU, ],
];

#[rustfmt::skip]
const INIT_MVD: [[u8; NUM_MV_RES_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 169,  198, ],
    [ 140,  198, ],
    [ CNU,  CNU, ],
];

#[rustfmt::skip]
const INIT_REF_PIC: [[u8; NUM_REF_NO_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 153,  153 ],
    [ 153,  153 ],
    [ CNU,  CNU ],
];

#[rustfmt::skip]
const INIT_DQP: [[u8; NUM_DELTA_QP_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 154,  154,  154, ],
    [ 154,  154,  154, ],
    [ 154,  154,  154, ],
];

#[rustfmt::skip]
const INIT_CHROMA_QP_ADJ_FLAG: [[u8; NUM_CHROMA_QP_ADJ_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 154, ],
    [ 154, ],
    [ 154, ],
];

#[rustfmt::skip]
const INIT_CHROMA_QP_ADJ_IDC: [[u8; NUM_CHROMA_QP_ADJ_IDC_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 154, ],
    [ 154, ],
    [ 154, ],
];

//  |---------Luminance---------| |--------Chrominance--------|
#[rustfmt::skip]
const INIT_QT_CBF: [[u8; NUM_QT_CBF_CTX_SETS * NUM_QT_CBF_CTX_PER_SET]; NUMBER_OF_SLICE_TYPES] = [
    [ 153,  111,  CNU,  CNU,  CNU, 149,   92,  167,  154,  154 ],
    [ 153,  111,  CNU,  CNU,  CNU, 149,  107,  167,  154,  154 ],
    [ 111,  141,  CNU,  CNU,  CNU,  94,  138,  182,  154,  154 ],
];

#[rustfmt::skip]
const INIT_QT_ROOT_CBF: [[u8; NUM_QT_ROOT_CBF_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [  79, ],
    [  79, ],
    [ CNU, ],
];

//  |------------------------------Luminance------------------------------| |------------------------------Chrominance--------------------------------|
#[rustfmt::skip]
const INIT_LAST: [[u8; NUM_CTX_LAST_FLAG_SETS * NUM_CTX_LAST_FLAG_XY]; NUMBER_OF_SLICE_TYPES] = [
    [ 125, 110, 124, 110,  95,  94, 125, 111, 111,  79, 125, 126, 111, 111,  79, 108, 123,  93, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU ],
    [ 125, 110,  94, 110,  95,  79, 125, 111, 110,  78, 110, 111, 111,  95,  94, 108, 123, 108, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU ],
    [ 110, 110, 124, 125, 140, 153, 125, 127, 140, 109, 111, 143, 127, 111,  79, 108, 123,  63, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU, CNU ],
];

//--------------------------------------------------------------------------------------------------
#[rustfmt::skip]
const INIT_SIG_CG_FLAG: [[u8; 2 * NUM_SIG_CG_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 121,  140,
       61,  154,
    ],
    [ 121,  140,
       61,  154,
    ],
    [  91,  171,
      134,  141,
    ],
];

//Initialisation for significance map

//  |-DC-|  |-----------------4x4------------------|  |------8x8 Diagonal Scan------|  |----8x8 Non-Diagonal Scan----|  |-NxN First group-|  |-NxN Other group-| |-Single context-|  |-DC-|  |-----------------4x4------------------|  |-8x8 Any group-|  |-NxN Any group-| |-Single context-|
//  |    |  |                                      |  |-First Group-| |-Other Group-|  |-First Group-| |-Other Group-|  |                 |  |                 | |                |
#[rustfmt::skip]
const INIT_SIG_FLAG: [[u8; NUM_SIG_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 170,    154, 139, 153, 139, 123, 123,  63, 124,   166, 183, 140,  136, 153, 154,   166, 183, 140,  136, 153, 154,   166,   183,   140,   136,   153,   154,        140,         170,    153, 138, 138, 122, 121, 122, 121, 167,   151,  183,  140,   151,  183,  140,        140 ],
    [ 155,    154, 139, 153, 139, 123, 123,  63, 153,   166, 183, 140,  136, 153, 154,   166, 183, 140,  136, 153, 154,   166,   183,   140,   136,   153,   154,        140,         170,    153, 123, 123, 107, 121, 107, 121, 167,   151,  183,  140,   151,  183,  140,        140 ],
    [ 111,    111, 125, 110, 110,  94, 124, 108, 124,   107, 125, 141,  179, 153, 125,   107, 125, 141,  179, 153, 125,   107,   125,   141,   179,   153,   125,        141,         140,    139, 182, 182, 152, 136, 152, 136, 153,   136,  139,  111,   136,  139,  111,        111 ],
];

//Initialisation for greater-than-one flags and greater-than-two flags

//------------------------------------------------
//   |------Set 0-------| |------Set 1-------| |------Set 2-------| |------Set 3-------| |------Set 4-------| |------Set 5-------|

#[rustfmt::skip]
const INIT_ONE_FLAG: [[u8; NUM_ONE_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 154, 196, 167, 167,  154, 152, 167, 182,  182, 134, 149, 136,  153, 121, 136, 122, 169, 208, 166, 167,  154, 152, 167, 182 ],
    [ 154, 196, 196, 167,  154, 152, 167, 182,  182, 134, 149, 136,  153, 121, 136, 137, 169, 194, 166, 167,  154, 167, 137, 182 ],
    [ 140,  92, 137, 138,  140, 152, 138, 139,  153,  74, 149,  92,  139, 107, 122, 152, 140, 179, 166, 182,  140, 227, 122, 197 ],
];

#[rustfmt::skip]
const INIT_ABS_FLAG: [[u8; NUM_ABS_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 107,                 167,                  91,                 107,                 107,                 167 ],
    [ 107,                 167,                  91,                 122,                 107,                 167 ],
    [ 138,                 153,                 136,                 167,                 152,                 152 ],
];

#[rustfmt::skip]
const INIT_MVP_IDX: [[u8; NUM_MVP_IDX_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 168, ],
    [ 168, ],
    [ CNU, ],
];

#[rustfmt::skip]
const INIT_SAO_MERGE_FLAG: [[u8; NUM_SAO_MERGE_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 153,  ],
    [ 153,  ],
    [ 153,  ],
];

#[rustfmt::skip]
const INIT_SAO_TYPE_IDX: [[u8; NUM_SAO_TYPE_IDX_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 160, ],
    [ 185, ],
    [ 200, ],
];

#[rustfmt::skip]
const INIT_TRANS_SUBDIV_FLAG: [[u8; NUM_TRANS_SUBDIV_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 224,  167,  122, ],
    [ 124,  138,   94, ],
    [ 153,  138,  138, ],
];

#[rustfmt::skip]
const INIT_TRANSFORMSKIP_FLAG: [[u8; 2*NUM_TRANSFORMSKIP_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 139,  139],
    [ 139,  139],
    [ 139,  139],
];

#[rustfmt::skip]
const INIT_EXPLICIT_RDPCM_FLAG: [[u8; 2*NUM_EXPLICIT_RDPCM_FLAG_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [139, 139],
    [139, 139],
    [CNU, CNU]
];

#[rustfmt::skip]
const INIT_EXPLICIT_RDPCM_DIR: [[u8; 2*NUM_EXPLICIT_RDPCM_DIR_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [139, 139],
    [139, 139],
    [CNU, CNU]
];

#[rustfmt::skip]
const INIT_CROSS_COMPONENT_PREDICTION: [[u8; NUM_CROSS_COMPONENT_PREDICTION_CTX]; NUMBER_OF_SLICE_TYPES] = [
    [ 154, 154, 154, 154, 154, 154, 154, 154, 154, 154 ],
    [ 154, 154, 154, 154, 154, 154, 154, 154, 154, 154 ],
    [ 154, 154, 154, 154, 154, 154, 154, 154, 154, 154 ],
];
