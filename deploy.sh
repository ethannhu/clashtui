#!/bin/bash
set -e

# Mihomo 一键安装脚本
# 基于 deploy_mihomo.md 文档

set -euo pipefail

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查是否以 root 权限运行
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "此脚本需要 root 权限运行，请使用 sudo"
        exit 1
    fi
}

# 检查系统
check_system() {
    if ! command -v systemctl &> /dev/null; then
        log_error "此脚本需要 systemd 系统，请确保您的系统支持 systemd"
        exit 1
    fi

    if ! command -v wget &> /dev/null && ! command -v curl &> /dev/null; then
        log_error "请安装 wget 或 curl 下载工具"
        exit 1
    fi

    log_info "系统检查通过"
}

# 获取系统架构
get_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64)
            echo "amd64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        armv7l)
            echo "armv7"
            ;;
        *)
            log_error "不支持的架构: $arch"
            exit 1
            ;;
    esac
}

# 获取系统类型
get_os() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "$os" in
        linux)
            echo "linux"
            ;;
        darwin)
            echo "darwin"
            ;;
        *)
            log_error "不支持的操作系统: $os"
            exit 1
            ;;
    esac
}

# 从本地安装 mihomo（查找当前目录下的 mihomo.gz）
install_local_mihomo() {
    local source_file="mihomo.gz"

    log_info "正在查找本地 mihomo.gz 文件..."

    # 查找当前目录下的 mihomo.gz
    if [[ ! -f "$source_file" ]]; then
        log_error "未在当前目录找到 mihomo.gz 文件"
        log_error "请确保 mihomo.gz 文件位于脚本同一目录下"
        exit 1
    fi

    log_info "找到 mihomo.gz，正在解压..."

    # 解压
    gunzip -f "$source_file" || {
        log_error "解压失败"
        exit 1
    }

    # 查找解压后的文件（可能是 mihomo 或 mihomo-linux-xxx）
    local binary_path=""
    if [[ -f "mihomo" ]]; then
        binary_path="mihomo"
    else
        # 查找其他可能的文件名
        for file in mihomo-linux-*; do
            if [[ -f "$file" ]]; then
                binary_path="$file"
                break
            fi
        done
    fi

    if [[ -z "$binary_path" || ! -f "$binary_path" ]]; then
        log_error "解压后未找到 mihomo 二进制文件"
        exit 1
    fi

    log_info "找到二进制文件: $binary_path"

    # 移动到目标位置
    log_info "正在安装 mihomo 到 /usr/local/bin/..."
    cp "$binary_path" /usr/local/bin/mihomo || {
        log_error "复制文件到 /usr/local/bin/ 失败"
        exit 1
    }

    chmod +x /usr/local/bin/mihomo || {
        log_error "设置可执行权限失败"
        exit 1
    }

    # 清理临时文件
    if [[ -f "$binary_path" && "$binary_path" != "/usr/local/bin/mihomo" ]]; then
        rm -f "$binary_path"
    fi

    log_info "mihomo 安装成功"
}

# 创建配置目录
create_config_dir() {
    log_info "正在创建配置目录 /etc/mihomo..."

    if [[ ! -d /etc/mihomo ]]; then
        mkdir -p /etc/mihomo || {
            log_error "创建 /etc/mihomo 目录失败"
            exit 1
        }
    else
        log_warn "目录 /etc/mihomo 已存在"
    fi

    # 复制配置文件（如果有的话）
    if [[ -f "config.yaml" ]]; then
        log_info "发现 config.yaml，正在复制到 /etc/mihomo/..."
        cp config.yaml /etc/mihomo/ || {
            log_error "复制配置文件失败"
            exit 1
        }
        log_info "配置文件复制成功"
    else
        log_warn "未找到 config.yaml 文件，将使用默认配置"
        # 创建最小配置文件
        cat > /etc/mihomo/config.yaml << 'EOF'
# Mihomo 配置文件
# 请根据需要修改此文件
EOF
    fi
}

# 创建 systemd 服务
create_systemd_service() {
    log_info "正在创建 systemd 服务..."

    local service_file="/etc/systemd/system/mihomo.service"

    if [[ -f "$service_file" ]]; then
        log_warn "服务文件已存在，将备份并重新创建"
        cp "$service_file" "${service_file}.bak.$(date +%Y%m%d%H%M%S)" || {
            log_error "备份服务文件失败"
            exit 1
        }
    fi

    cat > "$service_file" << 'EOF'
[Unit]
Description=mihomo Daemon, Another Clash Kernel.
After=network.target NetworkManager.service systemd-networkd.service iwd.service

[Service]
Type=simple
LimitNPROC=500
LimitNOFILE=1000000
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_RAW CAP_NET_BIND_SERVICE CAP_SYS_TIME CAP_SYS_PTRACE CAP_DAC_READ_SEARCH CAP_DAC_OVERRIDE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_RAW CAP_NET_BIND_SERVICE CAP_SYS_TIME CAP_SYS_PTRACE CAP_DAC_READ_SEARCH CAP_DAC_OVERRIDE
Restart=always
ExecStartPre=/usr/bin/sleep 1s
ExecStart=/usr/local/bin/mihomo -d /etc/mihomo
ExecReload=/bin/kill -HUP $MAINPID

[Install]
WantedBy=multi-user.target
EOF

    if [[ ! -f "$service_file" ]]; then
        log_error "创建服务文件失败"
        exit 1
    fi

    log_info "服务文件创建成功"
}

# 重载 systemd
reload_systemd() {
    log_info "正在重新加载 systemd..."

    systemctl daemon-reload || {
        log_error "systemd 重载失败"
        exit 1
    }

    log_info "systemd 重载成功"
}

# 启用并启动服务
enable_and_start_service() {
    log_info "正在启用 mihomo 服务..."

    systemctl enable mihomo || {
        log_error "启用服务失败"
        exit 1
    }

    log_info "正在启动 mihomo 服务..."

    systemctl start mihomo || {
        log_error "启动服务失败"
        exit 1
    }

    # 等待服务启动
    sleep 2

    # 检查服务状态
    if systemctl is-active --quiet mihomo; then
        log_info "mihomo 服务已成功启动"
    else
        log_warn "mihomo 服务可能未正常启动，请检查日志"
    fi
}

# 检查安装状态
check_status() {
    log_info "正在检查 mihomo 服务状态..."

    systemctl status mihomo --no-pager || true

    log_info ""
    log_info "查看日志命令: journalctl -u mihomo -o cat -f"
    log_info "重启服务命令: systemctl restart mihomo"
    log_info "停止服务命令: systemctl stop mihomo"
}

# 主函数
main() {
    echo "=========================================="
    echo "    Mihomo 一键安装脚本"
    echo "=========================================="
    echo ""

    check_root
    check_system

    log_info "开始安装 mihomo..."
    echo ""

    install_local_mihomo
    create_config_dir
    create_systemd_service
    reload_systemd
    enable_and_start_service

    echo ""
    echo "=========================================="
    log_info "mihomo 安装完成！"
    echo "=========================================="
    echo ""

    check_status
}

# 运行主函数
main "$@"
