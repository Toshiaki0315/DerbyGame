import type { OvalShape, Trace } from "./raceShow";

type Point = [number, number];
type Surface = "turf" | "dirt";

/**
 * The line of each course, read off the plans the JRA publishes (the coordinates are the
 * plan's pixels). Each starts at the goal and goes round against the running, so that
 * the way along it from the first point is what a horse has left to run.
 */
export const COURSE_SHAPES: Record<string, Point[]> = {
  "東京/turf/": [[400, 279], [349, 280], [299, 283], [248, 282], [197, 283], [147, 288], [96, 286], [52, 261], [29, 217], [27, 166], [41, 117], [73, 78], [119, 60], [170, 58], [221, 57], [272, 57], [322, 58], [373, 59], [423, 67], [465, 94], [492, 137], [500, 186], [486, 235], [450, 269]],
  "東京/dirt/": [[398, 260], [353, 261], [307, 262], [261, 264], [216, 262], [170, 262], [125, 261], [80, 253], [51, 219], [44, 175], [55, 130], [82, 94], [123, 77], [169, 76], [214, 76], [260, 75], [305, 75], [351, 75], [396, 80], [438, 97], [469, 130], [481, 174], [472, 218], [442, 251]],
  "中山/turf/内回り": [[234, 320], [277, 314], [321, 313], [365, 312], [409, 311], [452, 304], [489, 281], [510, 242], [512, 199], [494, 159], [458, 134], [417, 119], [373, 120], [329, 120], [285, 120], [241, 120], [197, 120], [155, 130], [120, 156], [102, 196], [99, 239], [113, 281], [148, 307], [190, 318]],
  "中山/turf/外回り": [[234, 320], [283, 314], [332, 312], [381, 314], [430, 309], [476, 292], [506, 253], [512, 205], [494, 160], [465, 121], [428, 88], [387, 61], [341, 43], [294, 29], [247, 15], [199, 18], [154, 38], [124, 77], [106, 123], [101, 171], [96, 219], [105, 267], [138, 302], [185, 318]],
  "中山/dirt/": [[234, 297], [272, 297], [311, 296], [349, 296], [387, 294], [426, 291], [461, 278], [484, 247], [490, 210], [477, 174], [449, 148], [411, 140], [373, 139], [334, 138], [296, 136], [258, 135], [220, 138], [181, 141], [147, 157], [123, 187], [118, 225], [130, 261], [158, 286], [195, 296]],
  "京都/turf/内回り": [[283, 207], [322, 207], [361, 206], [400, 206], [439, 204], [478, 198], [514, 185], [530, 150], [532, 111], [516, 76], [491, 46], [457, 27], [418, 27], [380, 34], [341, 40], [303, 48], [265, 55], [227, 63], [189, 74], [167, 104], [162, 142], [175, 178], [206, 202], [244, 208]],
  "京都/turf/外回り": [[283, 208], [323, 206], [363, 206], [403, 206], [444, 206], [483, 198], [518, 181], [533, 144], [536, 104], [534, 64], [512, 32], [474, 23], [434, 26], [394, 29], [354, 38], [315, 45], [275, 53], [236, 60], [198, 72], [169, 99], [162, 138], [173, 176], [204, 201], [243, 208]],
  "京都/dirt/": [[283, 194], [318, 192], [352, 193], [387, 192], [421, 192], [455, 186], [488, 176], [510, 149], [512, 115], [502, 82], [480, 55], [449, 42], [414, 41], [380, 47], [346, 54], [312, 60], [278, 68], [244, 73], [211, 83], [185, 106], [176, 138], [187, 171], [214, 191], [248, 195]],
  "阪神/turf/内回り": [[167, 287], [209, 287], [251, 287], [293, 288], [334, 288], [376, 287], [416, 276], [441, 243], [451, 203], [445, 161], [429, 123], [404, 89], [370, 65], [330, 62], [289, 73], [250, 88], [210, 100], [171, 114], [131, 127], [95, 147], [74, 183], [73, 224], [90, 262], [125, 283]],
  "阪神/turf/外回り": [[167, 287], [218, 288], [269, 287], [320, 289], [370, 287], [421, 281], [471, 276], [512, 246], [540, 204], [552, 155], [543, 105], [517, 62], [476, 33], [426, 30], [377, 45], [329, 60], [280, 76], [233, 92], [185, 109], [136, 126], [93, 150], [73, 196], [81, 246], [117, 280]],
  "阪神/dirt/": [[167, 271], [204, 271], [242, 271], [279, 271], [316, 272], [354, 271], [391, 265], [421, 245], [435, 211], [433, 174], [420, 139], [400, 107], [371, 84], [335, 76], [299, 86], [263, 97], [227, 107], [192, 119], [156, 130], [122, 145], [97, 172], [89, 208], [101, 243], [130, 266]],
  "中京/turf/": [[381, 231], [333, 230], [284, 230], [236, 233], [187, 233], [139, 231], [91, 228], [49, 205], [23, 165], [20, 117], [38, 73], [74, 40], [120, 26], [169, 28], [217, 32], [265, 36], [313, 43], [361, 46], [409, 55], [454, 71], [483, 109], [486, 157], [469, 202], [429, 227]],
  "中京/dirt/": [[381, 213], [338, 213], [295, 214], [251, 214], [208, 213], [165, 214], [122, 213], [80, 204], [49, 175], [36, 134], [47, 93], [76, 61], [116, 46], [159, 46], [203, 49], [246, 54], [289, 57], [332, 61], [375, 67], [417, 73], [452, 98], [467, 138], [456, 179], [424, 207]],
  "新潟/turf/内回り": [[430, 153], [400, 154], [371, 153], [341, 154], [311, 154], [282, 153], [253, 147], [234, 125], [226, 96], [234, 68], [253, 45], [282, 40], [311, 41], [341, 41], [371, 39], [400, 41], [430, 41], [460, 40], [489, 43], [506, 66], [513, 94], [508, 123], [488, 145], [460, 153]],
  "新潟/turf/外回り": [[430, 153], [390, 154], [350, 154], [310, 154], [271, 152], [231, 153], [191, 154], [151, 153], [116, 135], [102, 99], [113, 62], [147, 41], [186, 40], [226, 40], [266, 42], [306, 40], [346, 40], [386, 40], [426, 41], [466, 40], [500, 55], [513, 93], [503, 130], [470, 151]],
  "新潟/dirt/": [[430, 143], [404, 143], [377, 143], [351, 142], [325, 142], [298, 141], [272, 140], [250, 127], [239, 103], [242, 77], [259, 58], [284, 50], [311, 50], [337, 50], [364, 50], [390, 49], [416, 48], [442, 51], [468, 53], [491, 65], [502, 89], [498, 115], [482, 135], [457, 142]],
  "福島/turf/": [[205, 272], [257, 274], [308, 275], [360, 277], [412, 273], [462, 264], [503, 233], [526, 187], [524, 136], [495, 94], [451, 67], [402, 52], [350, 50], [298, 49], [246, 46], [195, 48], [143, 51], [93, 62], [55, 97], [33, 143], [34, 195], [58, 239], [102, 266], [153, 270]],
  "福島/dirt/": [[205, 256], [252, 256], [299, 256], [345, 255], [392, 257], [438, 253], [480, 233], [506, 194], [508, 148], [484, 108], [444, 85], [399, 73], [352, 74], [306, 75], [259, 74], [212, 73], [165, 74], [119, 78], [78, 100], [54, 139], [51, 185], [72, 226], [112, 250], [158, 256]],
  "小倉/turf/": [[215, 249], [268, 245], [320, 246], [373, 249], [426, 249], [476, 236], [516, 202], [536, 154], [532, 102], [504, 58], [459, 31], [407, 24], [354, 23], [301, 24], [249, 20], [196, 20], [143, 24], [93, 39], [52, 71], [32, 120], [37, 172], [65, 216], [110, 243], [162, 250]],
  "小倉/dirt/": [[215, 227], [262, 226], [308, 226], [355, 226], [402, 225], [448, 223], [488, 199], [511, 159], [510, 113], [486, 73], [446, 52], [400, 48], [353, 48], [306, 48], [260, 48], [213, 45], [166, 45], [121, 52], [80, 75], [58, 115], [58, 161], [82, 201], [122, 223], [168, 227]],
  "札幌/turf/": [[200, 289], [251, 289], [303, 290], [354, 288], [405, 286], [454, 270], [493, 238], [516, 192], [519, 141], [502, 93], [467, 55], [420, 35], [369, 31], [317, 32], [266, 34], [215, 31], [163, 32], [114, 46], [77, 81], [54, 127], [51, 178], [68, 225], [103, 264], [149, 285]],
  "札幌/dirt/": [[200, 269], [246, 269], [292, 269], [339, 267], [385, 269], [430, 261], [469, 236], [494, 197], [501, 152], [488, 108], [458, 73], [417, 53], [370, 49], [324, 49], [278, 52], [232, 50], [185, 52], [140, 58], [101, 82], [76, 121], [70, 167], [82, 211], [112, 246], [154, 265]],
  "函館/turf/": [[225, 253], [278, 252], [330, 253], [383, 251], [435, 249], [483, 228], [516, 188], [529, 138], [518, 87], [485, 46], [438, 24], [385, 20], [333, 20], [280, 18], [228, 17], [175, 19], [123, 22], [75, 43], [42, 83], [30, 134], [41, 185], [73, 226], [120, 248], [173, 251]],
  "函館/dirt/": [[228, 233], [276, 233], [323, 233], [371, 233], [419, 232], [464, 218], [497, 184], [510, 139], [500, 93], [469, 57], [425, 40], [377, 39], [330, 39], [282, 38], [234, 39], [187, 40], [139, 41], [94, 54], [60, 88], [47, 133], [58, 179], [89, 215], [133, 232], [180, 233]],
};

/** The gates that stand off the line of the course, read off the same plans. */
const GATES: Record<string, [number, number, string]> = {
  "東京/turf/1600": [434, 58, "向正面の奥"],
  "東京/turf/1800": [528, 160, "1〜2コーナー間のポケット"],
  "東京/turf/2000": [535, 240, "1コーナー奥ポケット"],
  "東京/dirt/1600": [487, 80, "2コーナー奥ポケット"],
  "中山/turf/1600": [85, 262, "1コーナー奥ポケット"],
  "中山/turf/2000": [432, 318, "正面スタンド前"],
  "中山/turf/2200": [445, 318, "正面スタンド前"],
  "中山/dirt/1200": [95, 133, "2コーナー奥ポケット"],
  "京都/turf/1600": [118, 88, "2コーナー奥ポケット"],
  "京都/turf/1800": [20, 103, "2コーナー奥ポケット"],
  "京都/turf/2400": [535, 205, "4コーナー奥ポケット"],
  "京都/turf/2200": [503, 205, "正面スタンド前"],
  "京都/dirt/1400": [118, 107, "2コーナー奥ポケット"],
  "阪神/turf/1200": [178, 108, "向正面"],
  "阪神/turf/1600": [178, 108, "向正面"],
  "阪神/turf/1800": [66, 140, "2コーナー奥ポケット"],
  "阪神/turf/1400": [66, 140, "2コーナー奥ポケット"],
  "阪神/turf/2200": [458, 288, "4コーナー奥ポケット"],
  "阪神/turf/2600": [458, 288, "4コーナー奥ポケット"],
  "阪神/dirt/1400": [20, 178, "2コーナー奥ポケット"],
  "阪神/dirt/2000": [470, 265, "4コーナー奥ポケット"],
  "中京/turf/1300": [403, 42, "向正面の奥"],
  "中京/turf/1400": [474, 48, "2コーナー奥ポケット"],
  "中京/turf/1600": [548, 210, "1コーナー奥ポケット"],
  "中京/turf/2200": [58, 232, "4コーナー奥ポケット"],
  "中京/dirt/1400": [532, 75, "2コーナー奥ポケット"],
  "新潟/turf/1200": [473, 38, "向正面の奥"],
  "新潟/turf/1800": [473, 38, "向正面の奥"],
  "新潟/turf/1400": [555, 38, "2コーナー奥ポケット"],
  "新潟/turf/2000": [555, 38, "2コーナー奥ポケット"],
  "新潟/dirt/1200": [518, 50, "2コーナー奥ポケット"],
  "福島/turf/1200": [40, 47, "2コーナー奥ポケット"],
  "福島/turf/2000": [515, 272, "4コーナー奥ポケット"],
  "福島/dirt/1150": [35, 77, "2コーナー奥ポケット"],
  "小倉/turf/1200": [35, 25, "2コーナー奥ポケット"],
  "小倉/turf/2000": [505, 255, "4コーナー奥ポケット"],
  "札幌/turf/1200": [95, 27, "2コーナー奥ポケット"],
  "札幌/turf/1500": [38, 283, "2コーナー奥ポケット"],
  "札幌/turf/2000": [460, 290, "4コーナー奥ポケット"],
  "函館/turf/1200": [30, 20, "2コーナー奥ポケット"],
  "函館/turf/2000": [520, 250, "4コーナー奥ポケット"],
};

/** The space left around a course in its drawing. */
const MARGIN = 16;
/** How many points of the drawn line go to each stretch of the plan's. */
const STEPS = 8;

export const shapeKey = (name: string, surface: Surface, layout: string | null | undefined): string =>
  `${name}/${surface}/${layout ?? ""}`;

/** Every point a course's drawing has to hold: its line and its gates off the line. */
const extentOf = (name: string, surface: Surface, line: Point[]) => {
  const gates = Object.entries(GATES)
    .filter(([key]) => key.startsWith(`${name}/${surface}/`))
    .map(([, [x, y]]) => [x, y] as Point);
  const all = [...line, ...gates];
  const xs = all.map(([x]) => x);
  const ys = all.map(([, y]) => y);
  return { minX: Math.min(...xs), maxX: Math.max(...xs), minY: Math.min(...ys), maxY: Math.max(...ys) };
};

/** What turns the plan's pixels into the drawing's. */
const fitter = (name: string, surface: Surface, line: Point[], width: number, height: number) => {
  const e = extentOf(name, surface, line);
  const scale = Math.min((width - 2 * MARGIN) / (e.maxX - e.minX), (height - 2 * MARGIN) / (e.maxY - e.minY));
  const left = (width - (e.maxX - e.minX) * scale) / 2;
  const top = (height - (e.maxY - e.minY) * scale) / 2;
  return ([x, y]: Point): Point => [left + (x - e.minX) * scale, top + (y - e.minY) * scale];
};

/** The plan's line smoothed into curves, closed back to the goal. */
const smooth = (line: Point[]): Point[] => {
  const at = (i: number) => line[(i + line.length) % line.length];
  const out: Point[] = [];
  line.forEach((_, i) => {
    const [p0, p1, p2, p3] = [at(i - 1), at(i), at(i + 1), at(i + 2)];
    for (let s = 0; s < STEPS; s++) {
      const t = s / STEPS;
      const blend = (a: number, b: number, c: number, d: number) =>
        0.5 * (2 * b + (c - a) * t + (2 * a - 5 * b + 4 * c - d) * t * t + (3 * b - a - 3 * c + d) * t * t * t);
      out.push([blend(p0[0], p1[0], p2[0], p3[0]), blend(p0[1], p1[1], p2[1], p3[1])]);
    }
  });
  return out;
};

/** A course's line fitted to a drawing, measured so a horse can be placed along it. */
export const traceOf = (
  name: string,
  surface: Surface,
  layout: string | null | undefined,
  width: number,
  height: number
): Trace | undefined => {
  const line = COURSE_SHAPES[shapeKey(name, surface, layout)];
  if (!line) return undefined;
  const fit = fitter(name, surface, line, width, height);
  const points = smooth(line).map(fit);
  const lengths = [0];
  points.forEach(([x, y], i) => {
    const [nx, ny] = points[(i + 1) % points.length];
    lengths.push(lengths[i] + Math.hypot(nx - x, ny - y));
  });
  const middle = points.reduce(([sx, sy], [x, y]) => [sx + x / points.length, sy + y / points.length], [0, 0]);
  return { points, lengths, total: lengths[points.length], middle: [middle[0], middle[1]] };
};

/** A gate that stands off the line of the course, in the drawing's terms. */
export const gateOf = (
  name: string,
  surface: Surface,
  distance: number,
  width: number,
  height: number,
  layout?: string | null
): { x: number; y: number; label: string } | undefined => {
  const gate = GATES[`${name}/${surface}/${distance}`];
  const line = COURSE_SHAPES[shapeKey(name, surface, layout)];
  if (!gate || !line) return undefined;
  const [x, y] = fitter(name, surface, line, width, height)([gate[0], gate[1]]);
  return { x, y, label: gate[2] };
};

/** Niigata's straight course: from its gate along the bottom of the outer course to the goal. */
const STRAIGHT_COURSES: Record<string, { from: Point; to: Point; alongside: string }> = {
  "新潟": { from: [15, 153], to: [430, 153], alongside: "新潟/turf/外回り" },
};

/** A straight course's line and the course it runs alongside, fitted to a drawing. */
const straightOf = (name: string, width: number, height: number) => {
  const course = STRAIGHT_COURSES[name];
  const line = course && COURSE_SHAPES[course.alongside];
  if (!course || !line) return undefined;
  const fit = fitter(name, "turf", [...line, course.from], width, height);
  return { line: { from: fit(course.from), to: fit(course.to) }, backdrop: smooth(line).map(fit) };
};

/** The course map's shape for a race. */
export const courseShape = (
  course: {
    name: string;
    turn: string;
    straight: number;
    lap: number;
    layout?: string | null;
    after_goal?: number;
    corners?: number;
    straight_course?: boolean;
  },
  surface: Surface,
  distance: number,
  width: number,
  height: number
): OvalShape => ({
  lap: course.lap,
  straight: course.straight,
  afterGoal: course.after_goal,
  corners: course.corners,
  straightCourse: course.straight_course,
  turn: course.turn,
  width,
  height,
  ...(course.straight_course ? straightOf(course.name, width, height) : {}),
  trace: course.straight_course ? undefined : traceOf(course.name, surface, course.layout, width, height),
  gate: course.straight_course ? undefined : gateOf(course.name, surface, distance, width, height, course.layout),
});
