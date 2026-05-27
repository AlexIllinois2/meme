// 共享类型定义

export interface Mode {
  id: number;
  name: string;
  sort_order: number;
  folder_path: string;
}

export interface Group {
  id: number;
  name: string;
  folder_path: string;
  share_count: number;
  mode_id: number;
  mode_name?: string;
}

export interface Image {
  id: number;
  image_path: string;
  thumbnail_path: string | null;
  share_count: number;
  group_id: number;
  mode_id: number;
}

export interface Config {
  meme_dir: string;
  color_mode: 'system' | 'light' | 'dark';
  theme_style: 'default' | 'modern' | 'minimal';
  last_mode: number;
  last_group: number;
  share_app: string;
  grid_size: number;
  pinyin_search: boolean;
  acronym_search: boolean;
  global_floating_window: boolean;
}


