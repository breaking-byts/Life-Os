// Deprecated: static exercise database was replaced by local SQLite cache + wger sync.

export interface StaticExercise {
  id: number
  name: string
  category: string
  muscles: string
  equipment: string
}

export const EXERCISE_DATABASE: StaticExercise[] = [
  // CHEST
  {
    id: 1,
    name: 'Bench Press',
    category: 'Chest',
    muscles: 'Pectorals, Triceps, Shoulders',
    equipment: 'Barbell',
  },
  {
    id: 2,
    name: 'Incline Bench Press',
    category: 'Chest',
    muscles: 'Upper Pectorals, Shoulders',
    equipment: 'Barbell',
  },
  {
    id: 3,
    name: 'Decline Bench Press',
    category: 'Chest',
    muscles: 'Lower Pectorals, Triceps',
    equipment: 'Barbell',
  },
  {
    id: 4,
    name: 'Dumbbell Bench Press',
    category: 'Chest',
    muscles: 'Pectorals, Triceps',
    equipment: 'Dumbbells',
  },
  {
    id: 5,
    name: 'Dumbbell Fly',
    category: 'Chest',
    muscles: 'Pectorals',
    equipment: 'Dumbbells',
  },
  {
    id: 6,
    name: 'Cable Crossover',
    category: 'Chest',
    muscles: 'Pectorals',
    equipment: 'Cable Machine',
  },
  {
    id: 7,
    name: 'Push-Up',
    category: 'Chest',
    muscles: 'Pectorals, Triceps, Core',
    equipment: 'Bodyweight',
  },
  {
    id: 8,
    name: 'Incline Dumbbell Press',
    category: 'Chest',
    muscles: 'Upper Pectorals',
    equipment: 'Dumbbells',
  },
  {
    id: 9,
    name: 'Chest Dip',
    category: 'Chest',
    muscles: 'Pectorals, Triceps',
    equipment: 'Dip Station',
  },
  {
    id: 10,
    name: 'Machine Chest Press',
    category: 'Chest',
    muscles: 'Pectorals, Triceps',
    equipment: 'Machine',
  },

  // BACK
  {
    id: 11,
    name: 'Deadlift',
    category: 'Back',
    muscles: 'Lower Back, Glutes, Hamstrings',
    equipment: 'Barbell',
  },
  {
    id: 12,
    name: 'Barbell Row',
    category: 'Back',
    muscles: 'Lats, Rhomboids, Biceps',
    equipment: 'Barbell',
  },
  {
    id: 13,
    name: 'Pull-Up',
    category: 'Back',
    muscles: 'Lats, Biceps, Core',
    equipment: 'Pull-Up Bar',
  },
  {
    id: 14,
    name: 'Chin-Up',
    category: 'Back',
    muscles: 'Lats, Biceps',
    equipment: 'Pull-Up Bar',
  },
  {
    id: 15,
    name: 'Lat Pulldown',
    category: 'Back',
    muscles: 'Lats, Biceps',
    equipment: 'Cable Machine',
  },
  {
    id: 16,
    name: 'Seated Cable Row',
    category: 'Back',
    muscles: 'Lats, Rhomboids',
    equipment: 'Cable Machine',
  },
  {
    id: 17,
    name: 'T-Bar Row',
    category: 'Back',
    muscles: 'Lats, Middle Back',
    equipment: 'T-Bar',
  },
  {
    id: 18,
    name: 'Dumbbell Row',
    category: 'Back',
    muscles: 'Lats, Biceps',
    equipment: 'Dumbbells',
  },
  {
    id: 19,
    name: 'Romanian Deadlift',
    category: 'Back',
    muscles: 'Lower Back, Hamstrings, Glutes',
    equipment: 'Barbell',
  },
  {
    id: 20,
    name: 'Face Pull',
    category: 'Back',
    muscles: 'Rear Delts, Upper Back',
    equipment: 'Cable Machine',
  },

  // SHOULDERS
  {
    id: 21,
    name: 'Overhead Press',
    category: 'Shoulders',
    muscles: 'Deltoids, Triceps',
    equipment: 'Barbell',
  },
  {
    id: 22,
    name: 'Dumbbell Shoulder Press',
    category: 'Shoulders',
    muscles: 'Deltoids, Triceps',
    equipment: 'Dumbbells',
  },
  {
    id: 23,
    name: 'Lateral Raise',
    category: 'Shoulders',
    muscles: 'Side Deltoids',
    equipment: 'Dumbbells',
  },
  {
    id: 24,
    name: 'Front Raise',
    category: 'Shoulders',
    muscles: 'Front Deltoids',
    equipment: 'Dumbbells',
  },
  {
    id: 25,
    name: 'Rear Delt Fly',
    category: 'Shoulders',
    muscles: 'Rear Deltoids',
    equipment: 'Dumbbells',
  },
  {
    id: 26,
    name: 'Arnold Press',
    category: 'Shoulders',
    muscles: 'Deltoids',
    equipment: 'Dumbbells',
  },
  {
    id: 27,
    name: 'Upright Row',
    category: 'Shoulders',
    muscles: 'Deltoids, Traps',
    equipment: 'Barbell',
  },
  {
    id: 28,
    name: 'Shrugs',
    category: 'Shoulders',
    muscles: 'Trapezius',
    equipment: 'Dumbbells',
  },
  {
    id: 29,
    name: 'Pike Push-Up',
    category: 'Shoulders',
    muscles: 'Deltoids, Triceps',
    equipment: 'Bodyweight',
  },
  {
    id: 30,
    name: 'Machine Shoulder Press',
    category: 'Shoulders',
    muscles: 'Deltoids',
    equipment: 'Machine',
  },

  // ARMS - BICEPS
  {
    id: 31,
    name: 'Barbell Curl',
    category: 'Arms',
    muscles: 'Biceps',
    equipment: 'Barbell',
  },
  {
    id: 32,
    name: 'Dumbbell Curl',
    category: 'Arms',
    muscles: 'Biceps',
    equipment: 'Dumbbells',
  },
  {
    id: 33,
    name: 'Hammer Curl',
    category: 'Arms',
    muscles: 'Biceps, Forearms',
    equipment: 'Dumbbells',
  },
  {
    id: 34,
    name: 'Preacher Curl',
    category: 'Arms',
    muscles: 'Biceps',
    equipment: 'Barbell',
  },
  {
    id: 35,
    name: 'Concentration Curl',
    category: 'Arms',
    muscles: 'Biceps',
    equipment: 'Dumbbells',
  },
  {
    id: 36,
    name: 'Cable Curl',
    category: 'Arms',
    muscles: 'Biceps',
    equipment: 'Cable Machine',
  },
  {
    id: 37,
    name: 'Incline Dumbbell Curl',
    category: 'Arms',
    muscles: 'Biceps',
    equipment: 'Dumbbells',
  },

  // ARMS - TRICEPS
  {
    id: 38,
    name: 'Tricep Pushdown',
    category: 'Arms',
    muscles: 'Triceps',
    equipment: 'Cable Machine',
  },
  {
    id: 39,
    name: 'Skull Crusher',
    category: 'Arms',
    muscles: 'Triceps',
    equipment: 'Barbell',
  },
  {
    id: 40,
    name: 'Overhead Tricep Extension',
    category: 'Arms',
    muscles: 'Triceps',
    equipment: 'Dumbbells',
  },
  {
    id: 41,
    name: 'Diamond Push-Up',
    category: 'Arms',
    muscles: 'Triceps, Chest',
    equipment: 'Bodyweight',
  },
  {
    id: 42,
    name: 'Tricep Dip',
    category: 'Arms',
    muscles: 'Triceps',
    equipment: 'Dip Station',
  },
  {
    id: 43,
    name: 'Close Grip Bench Press',
    category: 'Arms',
    muscles: 'Triceps, Chest',
    equipment: 'Barbell',
  },
  {
    id: 44,
    name: 'Kickback',
    category: 'Arms',
    muscles: 'Triceps',
    equipment: 'Dumbbells',
  },

  // LEGS
  {
    id: 45,
    name: 'Squat',
    category: 'Legs',
    muscles: 'Quadriceps, Glutes, Hamstrings',
    equipment: 'Barbell',
  },
  {
    id: 46,
    name: 'Front Squat',
    category: 'Legs',
    muscles: 'Quadriceps, Core',
    equipment: 'Barbell',
  },
  {
    id: 47,
    name: 'Leg Press',
    category: 'Legs',
    muscles: 'Quadriceps, Glutes',
    equipment: 'Machine',
  },
  {
    id: 48,
    name: 'Lunge',
    category: 'Legs',
    muscles: 'Quadriceps, Glutes',
    equipment: 'Dumbbells',
  },
  {
    id: 49,
    name: 'Bulgarian Split Squat',
    category: 'Legs',
    muscles: 'Quadriceps, Glutes',
    equipment: 'Dumbbells',
  },
  {
    id: 50,
    name: 'Leg Extension',
    category: 'Legs',
    muscles: 'Quadriceps',
    equipment: 'Machine',
  },
  {
    id: 51,
    name: 'Leg Curl',
    category: 'Legs',
    muscles: 'Hamstrings',
    equipment: 'Machine',
  },
  {
    id: 52,
    name: 'Hip Thrust',
    category: 'Legs',
    muscles: 'Glutes, Hamstrings',
    equipment: 'Barbell',
  },
  {
    id: 53,
    name: 'Calf Raise',
    category: 'Legs',
    muscles: 'Calves',
    equipment: 'Machine',
  },
  {
    id: 54,
    name: 'Goblet Squat',
    category: 'Legs',
    muscles: 'Quadriceps, Glutes',
    equipment: 'Dumbbells',
  },
  {
    id: 55,
    name: 'Step-Up',
    category: 'Legs',
    muscles: 'Quadriceps, Glutes',
    equipment: 'Dumbbells',
  },
  {
    id: 56,
    name: 'Hack Squat',
    category: 'Legs',
    muscles: 'Quadriceps',
    equipment: 'Machine',
  },
  {
    id: 57,
    name: 'Sumo Deadlift',
    category: 'Legs',
    muscles: 'Glutes, Adductors, Hamstrings',
    equipment: 'Barbell',
  },
  {
    id: 58,
    name: 'Walking Lunge',
    category: 'Legs',
    muscles: 'Quadriceps, Glutes',
    equipment: 'Dumbbells',
  },

  // CORE
  {
    id: 59,
    name: 'Plank',
    category: 'Core',
    muscles: 'Abs, Obliques',
    equipment: 'Bodyweight',
  },
  {
    id: 60,
    name: 'Crunch',
    category: 'Core',
    muscles: 'Abs',
    equipment: 'Bodyweight',
  },
  {
    id: 61,
    name: 'Sit-Up',
    category: 'Core',
    muscles: 'Abs, Hip Flexors',
    equipment: 'Bodyweight',
  },
  {
    id: 62,
    name: 'Bicycle Crunch',
    category: 'Core',
    muscles: 'Abs, Obliques',
    equipment: 'Bodyweight',
  },
  {
    id: 63,
    name: 'Leg Raise',
    category: 'Core',
    muscles: 'Lower Abs',
    equipment: 'Bodyweight',
  },
  {
    id: 64,
    name: 'Russian Twist',
    category: 'Core',
    muscles: 'Obliques',
    equipment: 'Bodyweight',
  },
  {
    id: 65,
    name: 'Mountain Climber',
    category: 'Core',
    muscles: 'Abs, Hip Flexors',
    equipment: 'Bodyweight',
  },
  {
    id: 66,
    name: 'Ab Wheel Rollout',
    category: 'Core',
    muscles: 'Abs, Core',
    equipment: 'Ab Wheel',
  },
  {
    id: 67,
    name: 'Dead Bug',
    category: 'Core',
    muscles: 'Core, Abs',
    equipment: 'Bodyweight',
  },
  {
    id: 68,
    name: 'Bird Dog',
    category: 'Core',
    muscles: 'Core, Lower Back',
    equipment: 'Bodyweight',
  },
  {
    id: 69,
    name: 'Side Plank',
    category: 'Core',
    muscles: 'Obliques',
    equipment: 'Bodyweight',
  },
  {
    id: 70,
    name: 'Hanging Knee Raise',
    category: 'Core',
    muscles: 'Lower Abs',
    equipment: 'Pull-Up Bar',
  },

  // CARDIO
  {
    id: 71,
    name: 'Running',
    category: 'Cardio',
    muscles: 'Full Body',
    equipment: 'None',
  },
  {
    id: 72,
    name: 'Cycling',
    category: 'Cardio',
    muscles: 'Legs, Heart',
    equipment: 'Bike',
  },
  {
    id: 73,
    name: 'Rowing',
    category: 'Cardio',
    muscles: 'Full Body',
    equipment: 'Rowing Machine',
  },
  {
    id: 74,
    name: 'Jump Rope',
    category: 'Cardio',
    muscles: 'Calves, Shoulders',
    equipment: 'Jump Rope',
  },
  {
    id: 75,
    name: 'Burpee',
    category: 'Cardio',
    muscles: 'Full Body',
    equipment: 'Bodyweight',
  },
  {
    id: 76,
    name: 'Box Jump',
    category: 'Cardio',
    muscles: 'Legs, Power',
    equipment: 'Box',
  },
  {
    id: 77,
    name: 'Kettlebell Swing',
    category: 'Cardio',
    muscles: 'Glutes, Hamstrings, Core',
    equipment: 'Kettlebell',
  },
  {
    id: 78,
    name: 'Battle Ropes',
    category: 'Cardio',
    muscles: 'Arms, Core',
    equipment: 'Battle Ropes',
  },
  {
    id: 79,
    name: 'Stair Climber',
    category: 'Cardio',
    muscles: 'Legs',
    equipment: 'Machine',
  },
  {
    id: 80,
    name: 'Elliptical',
    category: 'Cardio',
    muscles: 'Full Body',
    equipment: 'Machine',
  },

  // COMPOUND / OLYMPIC
  {
    id: 81,
    name: 'Clean and Jerk',
    category: 'Olympic',
    muscles: 'Full Body',
    equipment: 'Barbell',
  },
  {
    id: 82,
    name: 'Snatch',
    category: 'Olympic',
    muscles: 'Full Body',
    equipment: 'Barbell',
  },
  {
    id: 83,
    name: 'Power Clean',
    category: 'Olympic',
    muscles: 'Full Body',
    equipment: 'Barbell',
  },
  {
    id: 84,
    name: 'Thruster',
    category: 'Compound',
    muscles: 'Legs, Shoulders',
    equipment: 'Barbell',
  },
  {
    id: 85,
    name: 'Clean and Press',
    category: 'Compound',
    muscles: 'Full Body',
    equipment: 'Barbell',
  },
  {
    id: 86,
    name: 'Turkish Get-Up',
    category: 'Compound',
    muscles: 'Full Body, Core',
    equipment: 'Kettlebell',
  },
  {
    id: 87,
    name: "Farmer's Walk",
    category: 'Compound',
    muscles: 'Grip, Core, Traps',
    equipment: 'Dumbbells',
  },
  {
    id: 88,
    name: 'Muscle-Up',
    category: 'Compound',
    muscles: 'Back, Chest, Triceps',
    equipment: 'Pull-Up Bar',
  },
  {
    id: 89,
    name: 'Man Maker',
    category: 'Compound',
    muscles: 'Full Body',
    equipment: 'Dumbbells',
  },
  {
    id: 90,
    name: 'Devil Press',
    category: 'Compound',
    muscles: 'Full Body',
    equipment: 'Dumbbells',
  },

  // STRETCHING / MOBILITY
  {
    id: 91,
    name: 'Cat-Cow Stretch',
    category: 'Mobility',
    muscles: 'Spine, Core',
    equipment: 'Bodyweight',
  },
  {
    id: 92,
    name: 'Hip Flexor Stretch',
    category: 'Mobility',
    muscles: 'Hip Flexors',
    equipment: 'Bodyweight',
  },
  {
    id: 93,
    name: 'Pigeon Pose',
    category: 'Mobility',
    muscles: 'Glutes, Hip Flexors',
    equipment: 'Bodyweight',
  },
  {
    id: 94,
    name: 'Shoulder Dislocate',
    category: 'Mobility',
    muscles: 'Shoulders',
    equipment: 'Band',
  },
  {
    id: 95,
    name: "World's Greatest Stretch",
    category: 'Mobility',
    muscles: 'Full Body',
    equipment: 'Bodyweight',
  },
  {
    id: 96,
    name: 'Foam Rolling',
    category: 'Mobility',
    muscles: 'Various',
    equipment: 'Foam Roller',
  },
  {
    id: 97,
    name: 'Downward Dog',
    category: 'Mobility',
    muscles: 'Hamstrings, Calves, Shoulders',
    equipment: 'Bodyweight',
  },
  {
    id: 98,
    name: "Child's Pose",
    category: 'Mobility',
    muscles: 'Back, Hips',
    equipment: 'Bodyweight',
  },
  {
    id: 99,
    name: 'Couch Stretch',
    category: 'Mobility',
    muscles: 'Quadriceps, Hip Flexors',
    equipment: 'Bodyweight',
  },
  {
    id: 100,
    name: 'Band Pull-Apart',
    category: 'Mobility',
    muscles: 'Rear Delts, Upper Back',
    equipment: 'Band',
  },
]

// Search function with fuzzy matching
export function searchExercises(query: string): StaticExercise[] {
  if (!query || query.length < 2) return []

  const lowerQuery = query.toLowerCase()

  return EXERCISE_DATABASE.filter(
    (ex) =>
      ex.name.toLowerCase().includes(lowerQuery) ||
      ex.category.toLowerCase().includes(lowerQuery) ||
      ex.muscles.toLowerCase().includes(lowerQuery) ||
      ex.equipment.toLowerCase().includes(lowerQuery),
  ).slice(0, 20) // Limit results
}

// Get exercises by category
export function getExercisesByCategory(category: string): StaticExercise[] {
  return EXERCISE_DATABASE.filter(
    (ex) => ex.category.toLowerCase() === category.toLowerCase(),
  )
}

// Get all categories
export function getCategories(): string[] {
  return [...new Set(EXERCISE_DATABASE.map((ex) => ex.category))]
}
