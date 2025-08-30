// Alpine.js Global State Management for Ichimi Server Dashboard

// グローバルステートストア
window.IchimiStore = {
    // アプリケーション全体の状態
    state: Alpine.reactive({
        // サーバー情報
        server: {
            status: 'loading',
            version: '',
            uptime: 0,
            lastUpdate: null
        },
        
        // プロセスリスト
        processes: [],
        
        // UI状態
        ui: {
            searchQuery: '',
            selectedProcessId: null,
            autoRefreshEnabled: true,
            refreshInterval: 5000,
            theme: 'light'
        },
        
        // モーダル状態
        modals: {
            createProcess: {
                visible: false,
                form: {
                    id: '',
                    command: '',
                    args: [],
                    env: {},
                    cwd: '',
                    auto_start: false
                },
                errors: {}
            },
            viewLogs: {
                visible: false,
                processId: null,
                logs: [],
                stream: 'both',
                autoScroll: true,
                loading: false
            },
            confirmDelete: {
                visible: false,
                processId: null,
                processName: ''
            }
        },
        
        // 通知
        notifications: [],
        
        // エラー状態
        errors: {
            lastError: null,
            connectionError: false
        }
    }),
    
    // 算出プロパティ
    getters: {
        // フィルタリングされたプロセスリスト
        filteredProcesses() {
            const query = IchimiStore.state.ui.searchQuery.toLowerCase();
            if (!query) return IchimiStore.state.processes;
            
            return IchimiStore.state.processes.filter(p => {
                return p.info.id.toLowerCase().includes(query) ||
                       p.info.command.toLowerCase().includes(query) ||
                       (p.info.args && p.info.args.join(' ').toLowerCase().includes(query));
            });
        },
        
        // ステータス別カウント
        processStats() {
            const stats = {
                total: IchimiStore.state.processes.length,
                running: 0,
                stopped: 0,
                failed: 0,
                notStarted: 0
            };
            
            IchimiStore.state.processes.forEach(p => {
                switch (p.info.state.state) {
                    case 'Running':
                        stats.running++;
                        break;
                    case 'Stopped':
                        stats.stopped++;
                        break;
                    case 'Failed':
                        stats.failed++;
                        break;
                    case 'NotStarted':
                        stats.notStarted++;
                        break;
                }
            });
            
            return stats;
        },
        
        // 選択されたプロセス
        selectedProcess() {
            const id = IchimiStore.state.ui.selectedProcessId;
            if (!id) return null;
            return IchimiStore.state.processes.find(p => p.info.id === id);
        }
    },
    
    // アクション
    actions: {
        // 初期化
        async init() {
            await this.loadServerInfo();
            await this.loadProcesses();
            this.startAutoRefresh();
        },
        
        // サーバー情報の読み込み
        async loadServerInfo() {
            try {
                const response = await fetch('/api/status');
                if (response.ok) {
                    const data = await response.json();
                    IchimiStore.state.server = {
                        ...data,
                        status: 'running',
                        lastUpdate: new Date()
                    };
                    IchimiStore.state.errors.connectionError = false;
                } else {
                    throw new Error('サーバー情報の取得に失敗しました');
                }
            } catch (error) {
                console.error('Failed to load server info:', error);
                IchimiStore.state.errors.connectionError = true;
                this.showNotification('サーバーへの接続に失敗しました', 'error');
            }
        },
        
        // プロセスリストの読み込み
        async loadProcesses() {
            try {
                const response = await fetch('/api/processes');
                if (response.ok) {
                    IchimiStore.state.processes = await response.json();
                    IchimiStore.state.errors.connectionError = false;
                } else {
                    throw new Error('プロセスリストの取得に失敗しました');
                }
            } catch (error) {
                console.error('Failed to load processes:', error);
                IchimiStore.state.errors.connectionError = true;
                this.showNotification('プロセスリストの取得に失敗しました', 'error');
            }
        },
        
        // プロセスの作成
        async createProcess() {
            const modal = IchimiStore.state.modals.createProcess;
            modal.errors = {};
            
            // バリデーション
            if (!modal.form.id) {
                modal.errors.id = 'IDは必須です';
                return;
            }
            if (!modal.form.command) {
                modal.errors.command = 'コマンドは必須です';
                return;
            }
            
            // args を文字列から配列に変換
            const argsArray = modal.form.args && typeof modal.form.args === 'string' 
                ? modal.form.args.split(' ').filter(a => a.trim()) 
                : modal.form.args || [];
            
            // env を文字列からオブジェクトに変換
            const envObject = {};
            if (modal.form.env && typeof modal.form.env === 'string') {
                modal.form.env.split('\n').forEach(line => {
                    const [key, ...valueParts] = line.split('=');
                    if (key && valueParts.length > 0) {
                        envObject[key.trim()] = valueParts.join('=').trim();
                    }
                });
            } else if (typeof modal.form.env === 'object') {
                Object.assign(envObject, modal.form.env);
            }
            
            const payload = {
                id: modal.form.id,
                command: modal.form.command,
                args: argsArray,
                env: envObject,
                cwd: modal.form.cwd || null,
                auto_start: modal.form.auto_start
            };
            
            try {
                const response = await fetch('/api/processes', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload)
                });
                
                if (response.ok) {
                    this.closeCreateModal();
                    await this.loadProcesses();
                    this.showNotification(`プロセス '${payload.id}' を作成しました`, 'success');
                } else {
                    const error = await response.text();
                    this.showNotification(`エラー: ${error}`, 'error');
                }
            } catch (error) {
                console.error('Failed to create process:', error);
                this.showNotification('プロセスの作成に失敗しました', 'error');
            }
        },
        
        // プロセスの開始
        async startProcess(id) {
            try {
                const response = await fetch(`/api/processes/${id}/start`, {
                    method: 'POST'
                });
                
                if (response.ok) {
                    await this.loadProcesses();
                    this.showNotification(`プロセス '${id}' を開始しました`, 'success');
                } else {
                    const error = await response.text();
                    this.showNotification(`エラー: ${error}`, 'error');
                }
            } catch (error) {
                console.error('Failed to start process:', error);
                this.showNotification('プロセスの開始に失敗しました', 'error');
            }
        },
        
        // プロセスの停止
        async stopProcess(id) {
            try {
                const response = await fetch(`/api/processes/${id}/stop`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ grace_period_ms: 5000 })
                });
                
                if (response.ok) {
                    await this.loadProcesses();
                    this.showNotification(`プロセス '${id}' を停止しました`, 'success');
                } else {
                    const error = await response.text();
                    this.showNotification(`エラー: ${error}`, 'error');
                }
            } catch (error) {
                console.error('Failed to stop process:', error);
                this.showNotification('プロセスの停止に失敗しました', 'error');
            }
        },
        
        // プロセスの削除
        async deleteProcess(id) {
            const modal = IchimiStore.state.modals.confirmDelete;
            modal.processId = id;
            modal.processName = id;
            modal.visible = true;
        },
        
        // プロセスの削除実行
        async confirmDeleteProcess() {
            const id = IchimiStore.state.modals.confirmDelete.processId;
            
            try {
                const response = await fetch(`/api/processes/${id}`, {
                    method: 'DELETE'
                });
                
                if (response.ok) {
                    this.closeConfirmDeleteModal();
                    await this.loadProcesses();
                    this.showNotification(`プロセス '${id}' を削除しました`, 'success');
                } else {
                    const error = await response.text();
                    this.showNotification(`エラー: ${error}`, 'error');
                }
            } catch (error) {
                console.error('Failed to delete process:', error);
                this.showNotification('プロセスの削除に失敗しました', 'error');
            }
        },
        
        // ログの表示
        async viewLogs(id) {
            const modal = IchimiStore.state.modals.viewLogs;
            modal.processId = id;
            modal.visible = true;
            modal.loading = true;
            await this.loadLogs();
        },
        
        // ログの読み込み
        async loadLogs() {
            const modal = IchimiStore.state.modals.viewLogs;
            if (!modal.processId) return;
            
            try {
                const response = await fetch(
                    `/api/processes/${modal.processId}/logs?stream=${modal.stream}&lines=1000`
                );
                
                if (response.ok) {
                    modal.logs = await response.json();
                } else {
                    modal.logs = ['ログの取得に失敗しました'];
                }
            } catch (error) {
                console.error('Failed to load logs:', error);
                modal.logs = ['ログの取得に失敗しました'];
            } finally {
                modal.loading = false;
            }
        },
        
        // ログストリームの変更
        async setLogStream(stream) {
            IchimiStore.state.modals.viewLogs.stream = stream;
            await this.loadLogs();
        },
        
        // モーダルを開く
        openCreateModal() {
            const modal = IchimiStore.state.modals.createProcess;
            modal.visible = true;
            modal.form = {
                id: '',
                command: '',
                args: '',
                env: '',
                cwd: '',
                auto_start: false
            };
            modal.errors = {};
        },
        
        // モーダルを閉じる
        closeCreateModal() {
            IchimiStore.state.modals.createProcess.visible = false;
        },
        
        closeLogsModal() {
            IchimiStore.state.modals.viewLogs.visible = false;
        },
        
        closeConfirmDeleteModal() {
            IchimiStore.state.modals.confirmDelete.visible = false;
        },
        
        // 通知の表示
        showNotification(message, type = 'info') {
            const notification = {
                id: Date.now(),
                message,
                type,
                timestamp: new Date()
            };
            
            IchimiStore.state.notifications.push(notification);
            
            // 3秒後に自動的に削除
            setTimeout(() => {
                this.removeNotification(notification.id);
            }, 3000);
        },
        
        // 通知の削除
        removeNotification(id) {
            const index = IchimiStore.state.notifications.findIndex(n => n.id === id);
            if (index > -1) {
                IchimiStore.state.notifications.splice(index, 1);
            }
        },
        
        // 自動更新の開始
        startAutoRefresh() {
            if (!IchimiStore.state.ui.autoRefreshEnabled) return;
            
            this.refreshInterval = setInterval(async () => {
                if (IchimiStore.state.ui.autoRefreshEnabled) {
                    await this.loadProcesses();
                    await this.loadServerInfo();
                }
            }, IchimiStore.state.ui.refreshInterval);
        },
        
        // 自動更新の停止
        stopAutoRefresh() {
            if (this.refreshInterval) {
                clearInterval(this.refreshInterval);
                this.refreshInterval = null;
            }
        },
        
        // 自動更新の切り替え
        toggleAutoRefresh() {
            IchimiStore.state.ui.autoRefreshEnabled = !IchimiStore.state.ui.autoRefreshEnabled;
            if (IchimiStore.state.ui.autoRefreshEnabled) {
                this.startAutoRefresh();
            } else {
                this.stopAutoRefresh();
            }
        }
    },
    
    // ヘルパー関数
    helpers: {
        // ステータスバッジのクラス
        getStatusBadgeClass(state) {
            const classes = {
                'Running': 'badge-success',
                'Stopped': 'badge-warning',
                'Failed': 'badge-danger',
                'NotStarted': 'badge-secondary'
            };
            return classes[state.state] || 'badge-secondary';
        },
        
        // ステータステキスト
        getStatusText(state) {
            const texts = {
                'Running': '実行中',
                'Stopped': '停止',
                'Failed': '失敗',
                'NotStarted': '未開始'
            };
            return texts[state.state] || state.state;
        },
        
        // 日付フォーマット
        formatDate(date) {
            if (!date) return '-';
            const d = new Date(date);
            if (isNaN(d.getTime())) return '-';
            
            const pad = (n) => n.toString().padStart(2, '0');
            return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ` +
                   `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
        },
        
        // 相対時間の計算
        getRelativeTime(date) {
            if (!date) return '';
            const seconds = Math.floor((new Date() - new Date(date)) / 1000);
            
            if (seconds < 60) return `${seconds}秒前`;
            if (seconds < 3600) return `${Math.floor(seconds / 60)}分前`;
            if (seconds < 86400) return `${Math.floor(seconds / 3600)}時間前`;
            return `${Math.floor(seconds / 86400)}日前`;
        }
    }
};

// Alpine.js コンポーネント定義
document.addEventListener('alpine:init', () => {
    // メインアプリケーションコンポーネント
    Alpine.data('ichimiApp', () => ({
        // ストアへの参照
        store: IchimiStore.state,
        getters: IchimiStore.getters,
        actions: IchimiStore.actions,
        helpers: IchimiStore.helpers,
        
        // 初期化
        init() {
            IchimiStore.actions.init();
        }
    }));
});