// FileFairy GUI JavaScript

class FileFairyGUI {
    constructor() {
        this.currentTab = 'watch';
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.setupDragAndDrop();
        this.setupKeyboardShortcuts();
        this.loadWatchFolders();
    }

    // 이벤트 리스너 설정
    setupEventListeners() {
        // 탭 전환
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                this.switchTab(e.target.dataset.tab);
            });
        });

        // 검색 기능
        const searchBtn = document.getElementById('search-btn');
        const searchInput = document.getElementById('search-input');
        
        searchBtn.addEventListener('click', () => this.performSearch());
        searchInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') this.performSearch();
        });
    }

    // 키보드 단축키 설정
    setupKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            // Ctrl + 1,2,3 으로 탭 전환
            if (e.ctrlKey) {
                switch(e.key) {
                    case '1':
                        e.preventDefault();
                        this.switchTab('watch');
                        break;
                    case '2':
                        e.preventDefault();
                        this.switchTab('search');
                        break;
                    case '3':
                        e.preventDefault();
                        this.switchTab('rename');
                        break;
                    case 'f':
                        e.preventDefault();
                        if (this.currentTab === 'search') {
                            document.getElementById('search-input').focus();
                        }
                        break;
                }
            }
            
            // ESC로 로딩 취소 (향후 구현)
            if (e.key === 'Escape') {
                this.hideLoading();
            }
        });
    }

    // 드래그 앤 드롭 설정
    setupDragAndDrop() {
        const dropZones = document.querySelectorAll('.drop-zone');
        
        dropZones.forEach(zone => {
            // 드래그 진입
            zone.addEventListener('dragenter', (e) => {
                e.preventDefault();
                zone.classList.add('dragover');
            });

            // 드래그 오버 효과
            zone.addEventListener('dragover', (e) => {
                e.preventDefault();
                zone.classList.add('dragover');
                
                // 드래그 중인 파일 정보 표시 (가능한 경우)
                const items = e.dataTransfer.items;
                if (items && items.length > 0) {
                    const fileCount = items.length;
                    const dropText = zone.querySelector('.drop-text strong');
                    if (dropText) {
                        dropText.textContent = `${fileCount}개 파일을 놓으세요`;
                    }
                }
            });

            // 드래그 벗어남
            zone.addEventListener('dragleave', (e) => {
                // 자식 요소로 이동할 때는 제거하지 않음
                if (!zone.contains(e.relatedTarget)) {
                    zone.classList.remove('dragover');
                    this.resetDropZoneText(zone);
                }
            });

            // 파일 드롭 처리
            zone.addEventListener('drop', (e) => {
                e.preventDefault();
                zone.classList.remove('dragover');
                zone.classList.add('processing');
                
                this.handleFileDrop(e, zone.id).finally(() => {
                    zone.classList.remove('processing');
                    this.resetDropZoneText(zone);
                });
            });
        });

        // 전체 문서에서 드래그앤드롭 기본 동작 방지
        document.addEventListener('dragover', (e) => e.preventDefault());
        document.addEventListener('drop', (e) => e.preventDefault());
    }

    // 드롭 존 텍스트 리셋
    resetDropZoneText(zone) {
        const dropText = zone.querySelector('.drop-text strong');
        if (dropText) {
            const tabName = this.currentTab;
            const textMap = {
                'watch': '폴더를 여기에 드래그하세요',
                'search': '파일이나 폴더를 여기에 드래그하세요',
                'rename': '파일이나 폴더를 여기에 드래그하세요'
            };
            dropText.textContent = textMap[tabName] || '파일을 여기에 드래그하세요';
        }
    }

    // 탭 전환
    switchTab(tabName) {
        // 현재 활성 탭 제거
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.remove('active');
        });

        // 새 탭 활성화
        document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
        document.getElementById(`${tabName}-tab`).classList.add('active');
        
        this.currentTab = tabName;
        
        // Python API에 현재 탭 전달
        if (window.pywebview && window.pywebview.api) {
            window.pywebview.api.set_current_tab(tabName);
        }
    }

    // 파일 드롭 처리
    async handleFileDrop(event, zoneId) {
        const files = Array.from(event.dataTransfer.files);
        
        if (files.length === 0) {
            this.showToast('파일이 선택되지 않았습니다', 'warning');
            return;
        }

        this.showLoading(`${files.length}개 파일 처리 중...`);

        try {
            const validFiles = [];
            const invalidFiles = [];

            // 파일 정보 수집 및 검증
            for (const file of files) {
                const fileInfo = {
                    name: file.name,
                    path: file.path || file.webkitRelativePath || '',
                    size: file.size,
                    type: file.type,
                    lastModified: file.lastModified
                };

                // 경로가 없는 경우 대체 방법 시도
                if (!fileInfo.path) {
                    // DataTransfer 객체에서 경로 추출 시도
                    if (event.dataTransfer.items) {
                        for (let item of event.dataTransfer.items) {
                            if (item.kind === 'file') {
                                const entry = item.webkitGetAsEntry();
                                if (entry && entry.fullPath) {
                                    fileInfo.path = entry.fullPath;
                                    break;
                                }
                            }
                        }
                    }
                }

                const response = await this.callPythonAPI('handle_drop', fileInfo);
                
                if (response.success) {
                    validFiles.push({...fileInfo, ...response.data});
                } else {
                    invalidFiles.push({file: fileInfo, error: response.error});
                }
            }

            // 유효한 파일들 처리
            if (validFiles.length > 0) {
                await this.processValidFiles(validFiles);
                this.showToast(`${validFiles.length}개 파일이 성공적으로 처리되었습니다`, 'success');
            }

            // 실패한 파일들 알림
            if (invalidFiles.length > 0) {
                console.warn('처리 실패한 파일들:', invalidFiles);
                this.showToast(`${invalidFiles.length}개 파일 처리에 실패했습니다`, 'warning');
            }

        } catch (error) {
            console.error('파일 드롭 처리 오류:', error);
            this.showToast('파일 처리 중 오류가 발생했습니다: ' + error.message, 'error');
        } finally {
            this.hideLoading();
        }
    }

    // 유효한 파일들을 현재 탭에 따라 처리
    async processValidFiles(validFiles) {
        switch (this.currentTab) {
            case 'watch':
                await this.processWatchFiles(validFiles);
                break;
            case 'search':
                await this.processSearchFiles(validFiles);
                break;
            case 'rename':
                await this.processRenameFiles(validFiles);
                break;
        }
    }

    // 감시 탭에서 파일 처리
    async processWatchFiles(files) {
        const folders = files.filter(file => file.is_directory);
        const nonFolders = files.filter(file => !file.is_directory);

        for (const folder of folders) {
            await this.addWatchFolder(folder.path);
        }

        if (nonFolders.length > 0) {
            this.showToast(`폴더만 감시할 수 있습니다. ${nonFolders.length}개 파일은 무시됩니다`, 'warning');
        }
    }

    // 검색 탭에서 파일 처리 (파일이 드롭되면 해당 파일에서 검색)
    async processSearchFiles(files) {
        // 검색 탭에서는 드롭된 파일들의 내용을 미리보기로 표시
        const searchResults = document.getElementById('search-results');
        
        // 지원되는 파일과 지원되지 않는 파일 분리
        const supportedFiles = files.filter(file => file.is_supported);
        const unsupportedFiles = files.filter(file => !file.is_supported);
        
        let html = `
            <div class="dropped-files-info">
                <h4>📁 드롭된 파일들 (${files.length}개)</h4>
                <p>검색어를 입력하여 이 파일들에서 검색하거나, 전체 검색을 수행하세요</p>
        `;

        if (supportedFiles.length > 0) {
            html += `
                <div class="supported-files">
                    <h5 style="color: #50C878; margin: 1rem 0 0.5rem 0;">✅ 검색 가능한 파일 (${supportedFiles.length}개)</h5>
                    <div class="file-list">
                        ${supportedFiles.map(file => `
                            <div class="file-item">
                                <span class="file-icon">${this.getFileIcon(file.name, file.is_directory)}</span>
                                <span class="file-name">${file.name}</span>
                                <span class="file-type-badge supported">${file.extension || 'DIR'}</span>
                                <span class="file-path">${file.path}</span>
                                ${file.size ? `<span style="font-size: 0.8rem; color: #6c757d;">${this.formatFileSize(file.size)}</span>` : ''}
                            </div>
                        `).join('')}
                    </div>
                </div>
            `;
        }

        if (unsupportedFiles.length > 0) {
            html += `
                <div class="unsupported-files">
                    <h5 style="color: #dc3545; margin: 1rem 0 0.5rem 0;">❌ 검색 불가능한 파일 (${unsupportedFiles.length}개)</h5>
                    <div class="file-list">
                        ${unsupportedFiles.map(file => `
                            <div class="file-item">
                                <span class="file-icon">${this.getFileIcon(file.name, file.is_directory)}</span>
                                <span class="file-name">${file.name}</span>
                                <span class="file-type-badge unsupported">${file.extension || 'DIR'}</span>
                                <span class="file-path">${file.path}</span>
                            </div>
                        `).join('')}
                    </div>
                </div>
            `;
        }

        html += `</div>`;
        searchResults.innerHTML = html;

        // 지원되는 파일이 있으면 검색 입력창에 포커스
        if (supportedFiles.length > 0) {
            document.getElementById('search-input').focus();
        }
    }

    // 파일명 변경 탭에서 파일 처리
    async processRenameFiles(files) {
        const filePaths = files.map(file => file.path);
        await this.getFilenameSuggestions(filePaths);
    }

    // 감시 폴더 목록 로드
    async loadWatchFolders() {
        try {
            const [foldersResponse, statsResponse] = await Promise.all([
                this.callPythonAPI('get_watch_folders'),
                this.callPythonAPI('get_watch_folder_stats')
            ]);
            
            if (foldersResponse.success && statsResponse.success) {
                this.displayWatchFolders(foldersResponse.data, statsResponse.data);
            } else {
                this.displayWatchFolders(foldersResponse.data || []);
            }
        } catch (error) {
            console.error('감시 폴더 로드 실패:', error);
        }
    }

    // 감시 폴더 추가
    async addWatchFolder(folderPath) {
        try {
            const response = await this.callPythonAPI('add_watch_folder', folderPath);
            if (response.success) {
                this.showToast(response.message, 'success');
                await this.loadWatchFolders();
            } else {
                this.showToast(response.error, 'error');
            }
        } catch (error) {
            this.showToast('폴더 추가 실패: ' + error.message, 'error');
        }
    }

    // 감시 폴더 제거
    async removeWatchFolder(folderPath) {
        try {
            const response = await this.callPythonAPI('remove_watch_folder', folderPath);
            if (response.success) {
                this.showToast(response.message, 'success');
                await this.loadWatchFolders();
            } else {
                this.showToast(response.error, 'error');
            }
        } catch (error) {
            this.showToast('폴더 제거 실패: ' + error.message, 'error');
        }
    }

    // 감시 폴더 목록 표시
    displayWatchFolders(folders, stats = []) {
        const listElement = document.getElementById('watch-folders-list');
        
        if (!folders || folders.length === 0) {
            listElement.innerHTML = '<div class="empty-state">아직 감시 중인 폴더가 없습니다</div>';
            return;
        }

        const getStats = (folder) => stats.find(s => s.path === folder) || {file_count: 0, exists: true};

        listElement.innerHTML = folders.map(folder => {
            const stat = getStats(folder);
            return `
                <div class="folder-item ${!stat.exists ? 'folder-missing' : ''}">
                    <div class="folder-info">
                        <span class="folder-icon">📁</span>
                        <span class="folder-path">${folder}</span>
                        <span class="folder-stats">${stat.file_count}개 파일</span>
                        <span class="folder-status ${stat.exists ? 'active' : 'missing'}">
                            ${stat.exists ? '🟢 활성' : '🔴 없음'}
                        </span>
                    </div>
                    <button class="remove-btn" onclick="gui.removeWatchFolder('${folder.replace(/\\/g, '\\\\')}')" title="감시 중지">
                        ❌ 제거
                    </button>
                </div>
            `;
        }).join('');
    }

    // 파일 검색
    async performSearch() {
        const query = document.getElementById('search-input').value.trim();
        
        if (!query) {
            this.showToast('검색어를 입력해주세요', 'warning');
            return;
        }

        this.showLoading(`"${query}" 검색 중...`);

        try {
            const response = await this.callPythonAPI('search_files', query);
            if (response.success) {
                this.displaySearchResults(response.data);
                this.showToast(`${response.data.total_count}개 파일에서 결과를 찾았습니다`, 'success');
            } else {
                this.showToast(response.error, 'error');
                document.getElementById('search-results').innerHTML = `
                    <div class="empty-state">❌ ${response.error}</div>
                `;
            }
        } catch (error) {
            this.showToast('검색 실패: ' + error.message, 'error');
        } finally {
            this.hideLoading();
        }
    }

    // 검색 결과 표시
    displaySearchResults(data) {
        const resultsElement = document.getElementById('search-results');
        const { results, total_count, query } = data;
        
        if (!results || results.length === 0) {
            resultsElement.innerHTML = `
                <div class="empty-state">
                    "${query}"에 대한 검색 결과가 없습니다
                </div>
            `;
            return;
        }

        let html = `
            <div class="search-summary">
                <h4>🔍 "${query}" 검색 결과 (${total_count}개)</h4>
            </div>
        `;

        html += results.map((result, index) => `
            <div class="search-result-item ${!result.exists ? 'file-missing' : ''}">
                <div class="result-header">
                    <span class="file-icon">${this.getFileIcon(result.file_name, false)}</span>
                    <div class="result-file-name">${result.file_name}</div>
                    ${result.size ? `<span class="file-size">${this.formatFileSize(result.size)}</span>` : ''}
                    ${!result.exists ? '<span class="missing-badge">❌ 파일 없음</span>' : ''}
                </div>
                <div class="result-file-path">📂 ${result.folder_path}</div>
                <div class="result-content">${this.highlightSearchTerm(this.truncateText(result.document, 300), query)}</div>
                ${result.exists ? `
                    <div class="result-actions">
                        <button onclick="gui.openFileLocation('${result.file_path}')" class="action-btn">📂 위치 열기</button>
                        <button onclick="gui.copyPath('${result.file_path}')" class="action-btn">📋 경로 복사</button>
                    </div>
                ` : ''}
            </div>
        `).join('');

        resultsElement.innerHTML = html;
    }

    // 검색어 하이라이트
    highlightSearchTerm(text, query) {
        if (!text || !query) return text;
        const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
        return text.replace(regex, '<mark>$1</mark>');
    }

    // 파일명 추천 요청
    async getFilenameSuggestions(filePaths) {
        this.showLoading(`${filePaths.length}개 파일 분석 중...`);

        try {
            const response = await this.callPythonAPI('get_filename_suggestions', filePaths);
            if (response.success) {
                this.displayRenameSuggestions(response.data);
                const { processed_count, skipped_count } = response.data;
                this.showToast(
                    `파일명 추천 완료: ${processed_count}개 처리${skipped_count > 0 ? `, ${skipped_count}개 건너뜀` : ''}`, 
                    'success'
                );
            } else {
                this.showToast(response.error, 'error');
                document.getElementById('rename-results').innerHTML = `
                    <div class="empty-state" style="color: #dc3545;">
                        ❌ ${response.error}
                    </div>
                `;
            }
        } catch (error) {
            this.showToast('파일명 추천 실패: ' + error.message, 'error');
            document.getElementById('rename-results').innerHTML = `
                <div class="empty-state" style="color: #dc3545;">
                    ❌ 오류 발생: ${error.message}
                </div>
            `;
        } finally {
            this.hideLoading();
        }
    }

    // 파일명 추천 결과 표시
    displayRenameSuggestions(responseData) {
        const resultsElement = document.getElementById('rename-results');
        
        if (!responseData || !responseData.results || responseData.results.length === 0) {
            resultsElement.innerHTML = '<div class="empty-state">추천할 파일명이 없습니다</div>';
            return;
        }

        const { results, processed_count, skipped_count, skipped_files } = responseData;

        let html = '';

        // 처리 통계 표시
        if (processed_count > 0 || skipped_count > 0) {
            html += `
                <div class="processing-summary">
                    <h4>📊 처리 결과</h4>
                    <p>✅ 처리 완료: ${processed_count}개 | ${skipped_count > 0 ? `⚠️ 건너뛴 파일: ${skipped_count}개` : ''}</p>
                    ${skipped_files && skipped_files.length > 0 ? 
                        `<details>
                            <summary>건너뛴 파일 목록</summary>
                            <ul>${skipped_files.map(file => `<li>${file}</li>`).join('')}</ul>
                        </details>` : ''
                    }
                </div>
            `;
        }

        // 각 파일의 추천 결과 표시
        html += results.map((suggestion, index) => `
            <div class="rename-item">
                <div class="rename-original">
                    ${this.getFileIcon(suggestion.original_filename, false)} 
                    ${suggestion.original_filename}
                    ${suggestion.file_size ? `<span style="font-size: 0.8rem; color: #6c757d; margin-left: 0.5rem;">(${this.formatFileSize(suggestion.file_size)})</span>` : ''}
                </div>
                
                <div class="rename-suggestion">
                    <div class="suggestion-content">
                        <span class="suggested-name">${suggestion.new_filename}</span>
                        <span class="confidence-indicator" title="AI 신뢰도">🎯</span>
                    </div>
                    <button class="apply-btn" onclick="gui.applyRename('${suggestion.file_path}', '${suggestion.new_filename}')">
                        적용
                    </button>
                </div>
                
                ${suggestion.key_chunks && suggestion.key_chunks.length > 0 ? `
                    <div class="analysis-content">
                        <details>
                            <summary><strong>🔍 분석된 내용</strong> (${suggestion.key_chunks.length}개 핵심 구간)</summary>
                            <div class="key-chunks">
                                ${suggestion.key_chunks.map((chunk, i) => `
                                    <div class="chunk-item">
                                        <span class="chunk-number">${i + 1}</span>
                                        <span class="chunk-text">${this.truncateText(chunk, 150)}</span>
                                    </div>
                                `).join('')}
                            </div>
                        </details>
                    </div>
                ` : ''}
                
                <div class="file-path-info">
                    <small>📂 ${suggestion.file_path}</small>
                </div>
            </div>
        `).join('');

        resultsElement.innerHTML = html;
    }

    // 파일명 변경 적용
    async applyRename(filePath, newName) {
        if (!confirm(`파일명을 "${newName}"으로 변경하시겠습니까?`)) return;
        
        this.showLoading('파일명 변경 중...');
        
        try {
            const response = await this.callPythonAPI('apply_filename_change', filePath, newName);
            if (response.success) {
                this.showToast(response.message, 'success');
                // 변경된 항목 업데이트
                this.updateRenamedItem(filePath, response.new_path);
            } else {
                this.showToast(response.error, 'error');
            }
        } catch (error) {
            this.showToast('파일명 변경 실패: ' + error.message, 'error');
        } finally {
            this.hideLoading();
        }
    }

    // 변경된 항목 UI 업데이트
    updateRenamedItem(oldPath, newPath) {
        const items = document.querySelectorAll('.rename-item');
        items.forEach(item => {
            const pathInfo = item.querySelector('.file-path-info small');
            if (pathInfo && pathInfo.textContent.includes(oldPath)) {
                const suggestion = item.querySelector('.rename-suggestion');
                suggestion.innerHTML = `
                    <div class="success-indicator">
                        ✅ 변경 완료: ${newPath.split('\\').pop()}
                    </div>
                `;
                item.style.background = '#d4edda';
                item.style.borderLeftColor = '#28a745';
            }
        });
    }

    // 파일 위치 열기
    openFileLocation(filePath) {
        // 브라우저에서는 직접 파일 탐색기를 열 수 없으므로 경로만 표시
        this.showToast(`파일 위치: ${filePath}`, 'info');
    }

    // 경로 복사
    async copyPath(filePath) {
        try {
            await navigator.clipboard.writeText(filePath);
            this.showToast('경로가 클립보드에 복사되었습니다', 'success');
        } catch (error) {
            // 클립보드 API가 지원되지 않는 경우
            this.showToast(`경로: ${filePath}`, 'info');
        }
    }

    // 도움말 표시/숨김
    showHelp() {
        document.getElementById('help-modal').classList.add('show');
    }

    hideHelp() {
        document.getElementById('help-modal').classList.remove('show');
    }

    // Python API 호출
    async callPythonAPI(method, ...args) {
        if (!window.pywebview || !window.pywebview.api) {
            throw new Error('Python API를 사용할 수 없습니다');
        }

        return await window.pywebview.api[method](...args);
    }

    // 유틸리티 함수들
    truncateText(text, maxLength) {
        if (!text) return '';
        return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
    }

    showLoading(message = '처리 중...') {
        const overlay = document.getElementById('loading-overlay');
        const text = overlay.querySelector('.loading-text');
        text.textContent = message;
        overlay.classList.add('show');
    }

    hideLoading() {
        document.getElementById('loading-overlay').classList.remove('show');
    }

    showToast(message, type = 'success') {
        const toast = document.getElementById('toast');
        toast.textContent = message;
        toast.className = `toast ${type} show`;
        
        setTimeout(() => {
            toast.classList.remove('show');
        }, 3000);
    }

    // 파일 크기를 읽기 쉬운 형태로 변환
    formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    // 파일 타입에 따른 아이콘 반환
    getFileIcon(fileName, isDirectory) {
        if (isDirectory) return '📁';
        
        const ext = fileName.split('.').pop().toLowerCase();
        const iconMap = {
            'pdf': '📄',
            'doc': '📝', 'docx': '📝',
            'xls': '📊', 'xlsx': '📊',
            'ppt': '📽️', 'pptx': '📽️',
            'txt': '📄', 'md': '📄',
            'jpg': '🖼️', 'jpeg': '🖼️', 'png': '🖼️', 'gif': '🖼️',
            'mp4': '🎬', 'avi': '🎬', 'mov': '🎬',
            'mp3': '🎵', 'wav': '🎵',
            'zip': '📦', 'rar': '📦', '7z': '📦'
        };
        
        return iconMap[ext] || '📄';
    }
}

// 페이지 로드 시 GUI 초기화
let gui;
document.addEventListener('DOMContentLoaded', () => {
    gui = new FileFairyGUI();
});
