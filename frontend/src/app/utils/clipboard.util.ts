/*
    if (navigator.clipboard) {
        await navigator.clipboard.writeText(value);
    } else {
        ClipboardUtil.copyMessage(value);
    }
    this.alertService.showInfo('panel-stream-editor.stream_link_copied_to_clipboard');
 */
export class ClipboardUtil {
    public static copyMessage(value: string): void {
        const selBox = document.createElement('textarea');
        selBox.style.position = 'fixed';
        selBox.style.left = '0';
        selBox.style.top = '0';
        selBox.style.opacity = '0';
        selBox.value = value;
        document.body.appendChild(selBox);
        selBox.focus();
        selBox.select();
        document.execCommand('copy');
        document.body.removeChild(selBox);
    }
    public static setClipboardValue(text: string, typeVal?: string | undefined): Promise<void> {
        const type = typeVal || "text/plain";
        const clipboardItemData = { [type]: text };
        const clipboardItem = new ClipboardItem(clipboardItemData);
        return window.navigator.clipboard.write([clipboardItem]);
    }
}
