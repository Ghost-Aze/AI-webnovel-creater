export interface UserProfile {
  id: string;
  display_name: string;
  preferred_language: string;
  created_at: string;
  updated_at: string;
  revision: number;
}

export interface UpdateUserProfileInput {
  display_name: string;
  preferred_language: string;
}

export interface UserPreferences {
  id: string;
  preferred_narrator: string;
  preferred_pov: string;
  chapter_length: number;
  scene_length: number;
  dialogue_density: number;
  prose_level: string;
  pacing: string;
  avoid_repetition: boolean;
  created_at: string;
  updated_at: string;
  revision: number;
}

export interface UpdateUserPreferencesInput {
  preferred_narrator: string;
  preferred_pov: string;
  chapter_length: number;
  scene_length: number;
  dialogue_density: number;
  prose_level: string;
  pacing: string;
  avoid_repetition: boolean;
}
