# aifo.py
# AI File Organizer CLI

import argparse
import datetime
import os
import shutil
import sys
from pathlib import Path
import yaml

# --- 설정 (Configuration) ---

# 규칙 파일의 기본 이름
RULES_FILE_NAME = "rules.yml"

# 로그 파일의 기본 이름
LOG_FILE_NAME = "aifo.log"

# --- AI 모듈 (Placeholder) ---

def get_ai_keywords(file_path: Path) -> str:
    """
    [AI 개발자 구현 영역]
    주어진 텍스트 파일(.txt, .md)의 내용을 분석하여 핵심 키워드를 반환합니다.
    현재는 실제 AI 모델 대신 더미(dummy) 데이터를 반환합니다.
    
    Args:
        file_path (Path): 분석할 파일의 경로
        
    Returns:
        str: 추출된 키워드를 '_'로 연결한 문자열 (예: "핵심키워드1_핵심키워드2")
    """
    # TODO: 실제 로컬 LLM 또는 키워드 추출 로직으로 교체해야 합니다.
    print(f"🧠 AI: '{file_path.name}' 파일에서 키워드 추출 시뮬레이션...")
    # 파일 크기나 내용에 따라 다른 키워드를 반환하는 간단한 예시
    try:
        if file_path.stat().st_size > 100:
            return "프로젝트_결과보고"
        else:
            return "간단메모_아이디어"
    except Exception:
        return "키워드추출실패"

# --- 규칙 관리 (Rule Management) ---

def get_default_rules():
    """기본 규칙을 담은 딕셔너리를 반환합니다."""
    return {
        'move': {
            '문서': ['.pdf', '.docx', '.pptx', '.hwp', '.txt', '.md'],
            '이미지': ['.jpg', '.jpeg', '.png', '.gif'],
            '데이터': ['.csv', '.xlsx', '.json'],
            '압축파일': ['.zip', '.7z', '.rar'],
            '기타': 'others'
        },
        'rename': [
            {
                'target_ext': ['.txt', '.md'],
                'pattern': '{{date_created}}_{{original_name}}{{ext}}'
            },
            {
                'target_ext': ['.jpg', '.png'],
                'pattern': 'IMG_{{date_created}}_{{original_name}}{{ext}}'
            }
        ],
        'rename_ai': {
            'target_ext': ['.txt', '.md'],
            'pattern': '{{original_name}}_{{keywords}}{{ext}}'
        },
        'exclude': [
            '.git', '.DS_Store', 'node_modules', 'venv', 
            RULES_FILE_NAME, LOG_FILE_NAME
        ]
    }

def load_rules():
    """현재 디렉토리에서 rules.yml 파일을 찾아 로드합니다. 없으면 기본 규칙을 사용합니다."""
    rules_path = Path(RULES_FILE_NAME)
    if not rules_path.is_file():
        print(f"'{RULES_FILE_NAME}' 파일을 찾을 수 없습니다. 기본 규칙을 사용합니다.")
        return get_default_rules()
    
    try:
        with open(rules_path, 'r', encoding='utf-8') as f:
            return yaml.safe_load(f)
    except Exception as e:
        print(f"오류: '{RULES_FILE_NAME}' 파일을 읽는 중 문제가 발생했습니다. ({e})")
        sys.exit(1)

# --- 핵심 로직 (Core Logic) ---

def organize_files(target_path: Path, rules: dict, dry_run: bool, recursive: bool, use_ai: bool, skip_confirm: bool, log_file: str):
    """파일 정리 작업을 수행합니다."""
    
    # 1. 정리할 파일 목록 스캔
    print(f"🔍 '{target_path.resolve()}' 경로를 스캔합니다...")
    files_to_organize = []
    if recursive:
        file_iterator = target_path.rglob('*')
    else:
        file_iterator = target_path.glob('*')
    
    for item in file_iterator:
        # 제외 목록에 있는 파일/폴더명은 건너뜀
        if item.name in rules.get('exclude', []):
            continue
        if item.is_file():
            files_to_organize.append(item)

    if not files_to_organize:
        print("정리할 파일이 없습니다.")
        return

    # 2. 변경 계획 수립
    change_plan = []
    for old_path in files_to_organize:
        new_name = old_path.name
        new_dir = old_path.parent

        # 2-1. 이름 변경 계획 (Rename)
        original_name = old_path.stem
        ext = old_path.suffix
        date_created = datetime.datetime.fromtimestamp(old_path.stat().st_ctime).strftime('%Y-%m-%d')
        
        # AI 키워드 사용 시
        if use_ai and ext in rules.get('rename_ai', {}).get('target_ext', []):
            keywords = get_ai_keywords(old_path)
            pattern = rules['rename_ai'].get('pattern', '{{original_name}}_{{keywords}}{{ext}}')
            new_name = pattern.replace('{{original_name}}', original_name)\
                              .replace('{{keywords}}', keywords)\
                              .replace('{{ext}}', ext)
        else: # 일반 이름 변경 규칙 적용
            for rule in rules.get('rename', []):
                if ext in rule.get('target_ext', []):
                    pattern = rule.get('pattern', '{{original_name}}{{ext}}')
                    new_name = pattern.replace('{{original_name}}', original_name)\
                                      .replace('{{date_created}}', date_created)\
                                      .replace('{{ext}}', ext)
                    break

        # 2-2. 폴더 이동 계획 (Move)
        moved = False
        for dir_name, exts in rules.get('move', {}).items():
            if exts == 'others': continue
            if ext in exts:
                new_dir = target_path / dir_name
                moved = True
                break
        
        # 'others' 규칙 처리
        if not moved and 'others' in rules.get('move', {}):
            dir_name = rules['move']['others']
            if isinstance(dir_name, str):
                new_dir = target_path / dir_name

        new_path = new_dir / new_name
        
        # 변경 사항이 있을 경우에만 계획에 추가
        if old_path != new_path:
            change_plan.append({'from': old_path, 'to': new_path})

    # 3. 결과 출력 및 실행
    if not change_plan:
        print("✅ 모든 파일이 이미 정리되어 있습니다. 변경할 내용이 없습니다.")
        return

    print("\n✨ 다음과 같이 파일이 변경될 예정입니다:\n")
    for change in change_plan:
        print(f"  - [이동/변경] '{change['from'].relative_to(target_path)}' -> '{change['to'].relative_to(target_path)}'")

    if dry_run:
        print("\n💧 Dry-run 모드입니다. 실제 파일은 변경되지 않았습니다.")
        return

    # 4. 사용자 확인
    if not skip_confirm:
        confirm = input(f"\n총 {len(change_plan)}개의 파일을 변경하시겠습니까? (y/n): ")
        if confirm.lower() != 'y':
            print("작업을 취소했습니다.")
            return

    # 5. 실제 파일 변경 실행
    print("\n🚀 파일 정리를 시작합니다...")
    log_entries = []
    for change in change_plan:
        try:
            # 이동할 디렉토리가 없으면 생성
            change['to'].parent.mkdir(parents=True, exist_ok=True)
            # 파일 이동/이름변경
            shutil.move(str(change['from']), str(change['to']))
            log_entry = f"[{datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] MOVED: '{change['from']}' -> '{change['to']}'"
            print(f"  - 성공: {change['from'].name} -> {change['to'].relative_to(target_path)}")
            log_entries.append(log_entry)
        except Exception as e:
            log_entry = f"[{datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] FAILED: '{change['from']}' -> '{change['to']}. 이유: {e}"
            print(f"  - 실패: {change['from'].name}. 이유: {e}")
            log_entries.append(log_entry)

    # 6. 로그 기록
    if log_file:
        log_path = Path(log_file)
        with open(log_path, 'a', encoding='utf-8') as f:
            for entry in log_entries:
                f.write(entry + '\n')
        print(f"\n📝 작업 결과가 '{log_path.resolve()}' 파일에 기록되었습니다.")

    print("\n🎉 파일 정리가 완료되었습니다!")


def scan_files(target_path: Path, recursive: bool, by_ext: bool, by_date: bool):
    """폴더 현황을 분석하여 리포트를 출력합니다."""
    print(f"📊 '{target_path.resolve()}' 경로를 분석합니다...")
    
    files = []
    if recursive:
        file_iterator = target_path.rglob('*')
    else:
        file_iterator = target_path.glob('*')

    for item in file_iterator:
        if item.is_file():
            files.append(item)
    
    if not files:
        print("분석할 파일이 없습니다.")
        return

    print(f"\n총 {len(files)}개의 파일을 찾았습니다.")

    if by_ext:
        print("\n--- 확장자별 파일 현황 ---")
        ext_count = {}
        for file in files:
            ext = file.suffix or '.no_extension'
            ext_count[ext] = ext_count.get(ext, 0) + 1
        
        # 개수 기준으로 정렬
        sorted_exts = sorted(ext_count.items(), key=lambda item: item[1], reverse=True)
        
        for ext, count in sorted_exts:
            print(f"  - {ext:<15}: {count}개")

    if by_date:
        print("\n--- 수정일 기준 파일 현황 (가장 오래된 파일 5개) ---")
        files.sort(key=lambda f: f.stat().st_mtime)
        for file in files[:5]:
            mtime = datetime.datetime.fromtimestamp(file.stat().st_mtime).strftime('%Y-%m-%d %H:%M')
            print(f"  - [{mtime}] {file.name}")

# --- CLI 인터페이스 (CLI Interface) ---

def main():
    parser = argparse.ArgumentParser(
        prog="aifo",
        description="🤖 AI 파일 정리 에이전트: 규칙과 AI를 사용하여 파일을 정리합니다.",
        epilog="자세한 사용법은 'aifo [COMMAND] --help'를 입력하세요."
    )
    subparsers = parser.add_subparsers(dest="command", required=True, help="수행할 작업")

    # 'organize' 명령어
    parser_organize = subparsers.add_parser("organize", help="규칙에 따라 파일을 정리합니다.")
    parser_organize.add_argument("target_path", type=Path, help="정리할 폴더 경로")
    parser_organize.add_argument("-R", "--rules", type=Path, default=Path(RULES_FILE_NAME), help=f"사용자 정의 규칙 파일 경로 (기본값: {RULES_FILE_NAME})")
    parser_organize.add_argument("-d", "--dry-run", action="store_true", help="[안전 기능] 실제 파일 변경 없이 시뮬레이션 결과만 보여줍니다.")
    parser_organize.add_argument("-r", "--recursive", action="store_true", help="하위 폴더를 포함하여 전체를 정리합니다.")
    parser_organize.add_argument("-a", "--ai-keyword", action="store_true", help="[AI 기능] 문서 파일 내용 기반으로 키워드를 추출하여 파일명에 추가합니다.")
    parser_organize.add_argument("-y", "--yes", action="store_true", help="[주의] 파일 변경 전 확인 절차를 생략하고 즉시 실행합니다.")
    parser_organize.add_argument("-l", "--log", nargs='?', const=LOG_FILE_NAME, default=None, help=f"작업 로그를 파일로 기록합니다. 파일명을 지정하지 않으면 '{LOG_FILE_NAME}'에 저장됩니다.")

    # 'scan' 명령어
    parser_scan = subparsers.add_parser("scan", help="폴더 현황을 분석하여 리포트를 보여줍니다.")
    parser_scan.add_argument("target_path", type=Path, help="분석할 폴더 경로")
    parser_scan.add_argument("-r", "--recursive", action="store_true", help="하위 폴더를 포함하여 전체를 분석합니다.")
    parser_scan.add_argument("-e", "--by-ext", action="store_true", help="확장자별로 파일 개수를 보여줍니다.")
    parser_scan.add_argument("-t", "--by-date", action="store_true", help="수정일 기준으로 파일을 정렬하여 보여줍니다.")
    
    # 'config' 명령어
    parser_config = subparsers.add_parser("config", help="규칙 설정 파일을 관리합니다.")
    config_subparsers = parser_config.add_subparsers(dest="config_command", required=True)
    config_subparsers.add_parser("init", help=f"현재 폴더에 예시 '{RULES_FILE_NAME}' 파일을 생성합니다.")
    config_subparsers.add_parser("show", help=f"현재 적용될 '{RULES_FILE_NAME}' 파일의 내용을 보여줍니다.")
    config_subparsers.add_parser("edit", help=f"기본 편집기로 '{RULES_FILE_NAME}' 파일을 엽니다 (경로 출력).")

    args = parser.parse_args()

    # 명령어 실행
    if args.command == "organize":
        if not args.target_path.is_dir():
            print(f"오류: '{args.target_path}'는 유효한 디렉토리가 아닙니다.")
            sys.exit(1)
        rules = load_rules()
        organize_files(args.target_path, rules, args.dry_run, args.recursive, args.ai_keyword, args.yes, args.log)
    
    elif args.command == "scan":
        if not args.target_path.is_dir():
            print(f"오류: '{args.target_path}'는 유효한 디렉토리가 아닙니다.")
            sys.exit(1)
        scan_files(args.target_path, args.recursive, args.by_ext, args.by_date)
        
    elif args.command == "config":
        rules_path = Path(RULES_FILE_NAME)
        if args.config_command == "init":
            if rules_path.exists():
                print(f"'{rules_path}' 파일이 이미 존재합니다.")
            else:
                with open(rules_path, 'w', encoding='utf-8') as f:
                    yaml.dump(get_default_rules(), f, allow_unicode=True, sort_keys=False, indent=2)
                print(f"✅ 예시 규칙 파일 '{rules_path.resolve()}'을 생성했습니다.")
        
        elif args.config_command == "show":
            if not rules_path.exists():
                print(f"'{rules_path}' 파일이 없습니다. 'aifo config init' 명령어로 생성할 수 있습니다.")
            else:
                print(f"--- {rules_path.resolve()} 내용 ---")
                print(rules_path.read_text(encoding='utf-8'))
                
        elif args.config_command == "edit":
            print(f"아래 경로의 파일을 직접 열어서 수정해주세요:\n{rules_path.resolve()}")


if __name__ == "__main__":
    main()
