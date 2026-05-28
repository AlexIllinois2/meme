# 继续拆分 App.vue — 提取顺序

1. **useSearch** (~80 行) — 最简单独立
   - searchKeyword, searchInputRef, searchImages(), handleFloatingSearchClick()

2. **useEditMode** (~200 行) — 依赖较少
   - 编辑模式开关、选中状态、批量删除、首次使用引导

3. **useShare** (~250 行) — 中等复杂度
   - 分享目标、分享到应用/复制、onCustomAppSelected 全局回调

4. **CRUD 对话框** (~500 行) — 最大最复杂
   - 所有的弹窗、菜单、增删改逻辑
