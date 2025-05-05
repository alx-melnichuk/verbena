import {
    afterNextRender, AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, EventEmitter, inject, Injector, Input, OnChanges,
    Output, SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CdkTextareaAutosize } from '@angular/cdk/text-field';
import { CommonModule, KeyValue } from '@angular/common';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { DateAdapter } from '@angular/material/core';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatMenuModule } from '@angular/material/menu';
import { TranslatePipe } from '@ngx-translate/core';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { StringDateTime } from 'src/app/common/string-date-time';
import { ChatMsg } from 'src/app/lib-stream/stream-chats.interface';
import { DateUtil } from 'src/app/utils/date.utils';
import { ReplaceWithZeroUtil } from 'src/app/utils/replace-with-zero.util';
import { StringDateTimeUtil } from 'src/app/utils/string-date-time.util';

interface MenuData {
    isEdit: boolean;
    isRemove: boolean;
}

export const MIN_ROWS = 1;
export const MAX_ROWS = 3;

// <mat-form-field subscriptSizing="dynamic"
// it'll remove the space until an error or hint actually needs to get displayed and only then expands.

@Component({
    selector: 'app-panel-chat',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, MatMenuModule,
        TranslatePipe, DateTimeFormatPipe],
    templateUrl: './panel-chat.component.html',
    styleUrl: './panel-chat.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelChatComponent implements OnChanges, AfterViewInit {
    @Input()
    public chatMsgs: ChatMsg[] = [];
    @Input()
    public isEditable: boolean | null = null;
    @Input()
    public locale: string | null = null;
    @Input()
    public maxRows: number | null = null;
    @Input()
    public minRows: number | null = null;
    @Input()
    public nickname: string | null = null;
    @Input()
    public title = '';
    // -- old --
    @Input()
    public isStreamOwner: boolean | null = true;
    @Input()
    public isUserBanned: boolean | null = null;
    @Input()
    public bannedUserIds: string[] = [];

    @Output()
    readonly sendMsg: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly removeMsg: EventEmitter<KeyValue<number, string>> = new EventEmitter();
    @Output()
    readonly editMsg: EventEmitter<KeyValue<number, string>> = new EventEmitter();
    //   @Output()
    //   readonly bannedUser: EventEmitter<string> = new EventEmitter();

    @ViewChild('autosize')
    public autosize!: CdkTextareaAutosize;
    @ViewChild('scrollItem')
    private scrollItemContainer!: ElementRef<HTMLElement>;
    @ViewChild('textareaElement')
    public textareaElem!: ElementRef<HTMLTextAreaElement>;

    public listChatMsg: ChatMsg[] = [];
    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ newMsg: this.formControl });
    public maxLen: number = 255;
    public maxRowsVal: number = MAX_ROWS;
    public minRowsVal: number = MIN_ROWS;
    public menuDataMap: Map<string, MenuData> = new Map();
    public msgMarked: ChatMsg | null = null;
    public msgEditing: ChatMsg | null = null;
    public initValue: string | null = null;

    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'medium' };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: 'short' };

    // ** OLD **
    // public isShowFaceSmilePanel = false;

    // public faceSmileList: string[] = [
    //     ...this.getEmojiPart0(),
    //     ...this.getEmojiPart2(),
    // ];

    private readonly dateAdapter: DateAdapter<Date> = inject(DateAdapter);
    private _injector = inject(Injector);

    constructor() {
        // this.chatMsgs = this.getChatMsg('evelyn_allen', 18); // #
    }

    triggerResize() {
        // Wait for content to render, then trigger textarea resize.
        afterNextRender(
            () => {
                this.autosize.resizeToFitContent(true);
            },
            {
                injector: this._injector,
            },
        );
    }
    ngAfterViewInit(): void {
        this.scrollToBottom();
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['maxRows']) {
            this.maxRowsVal = (!!this.maxRows && this.maxRows > 0 ? this.maxRows : MAX_ROWS);
        }
        if (!!changes['minRows']) {
            this.minRowsVal = (!!this.minRows && this.minRows > 0 ? this.minRows : MIN_ROWS);
        }
        if (!!changes['chatMsgs']) {
            console.log(`PanelChat.OnChange('chatMsgs') chatMsgs: ${JSON.stringify(this.chatMsgs)}`);
            this.listChatMsg.push(...this.chatMsgs);
            this.scrollToBottom();
            Promise.resolve().then(() =>
                this.scrollToBottom());
        }
    }

    // ** Public API **

    public trackById(index: number, item: ChatMsg): string {
        return item.date;
    }
    public memberWithZero(value: string): string {
        return ReplaceWithZeroUtil.replace(value);
    }
    public isEnableMenu(chatMsg: ChatMsg | null, selfName: string | null): boolean {
        return !!selfName && selfName == chatMsg?.member;
    }

    public getMenuDataByMap(chatMsg: ChatMsg | null): MenuData {
        let result: MenuData = this.getMenuData('_' + this.nickname, this.nickname || '');
        if (!!chatMsg && !!this.nickname) {
            result = this.getMenuData(chatMsg.member, this.nickname);
            this.menuDataMap.set(chatMsg.date, result);
        }
        return result;
    }
    public cleanNewMsg(): void {
        this.setTextareaValue(null);
        this.msgEditing = null;
    }

    public doSendMessage(newMsg: string): void {
        const newMsgVal = (newMsg || '').trim();
        if (this.isEditable && newMsgVal.length > 0) {
            if (!!this.msgEditing) {
                const keyValue: KeyValue<number, string> = { key: this.msgEditing.id, value: newMsgVal };
                this.editMsg.emit(keyValue);
            } else {
                this.sendMsg.emit(newMsgVal);
            }
            this.cleanNewMsg();
            this.scrollToBottom();
        }
    }
    public doRemoveMessage(chatMsg: ChatMsg | null): void {
        if (this.isEditable && !!chatMsg && !!chatMsg.date && chatMsg.member == this.nickname) {
            const keyValue: KeyValue<number, string> = { key: chatMsg.id, value: chatMsg.msg };
            this.removeMsg.emit(keyValue);
        }
    }
    public doSetValueForEditing(chatMsg: ChatMsg | null): void {
        if (this.msgEditing != chatMsg) {
            this.msgEditing = chatMsg;
            this.setTextareaValue(chatMsg?.msg || null);
            this.textareaElem.nativeElement.focus();
        }
    }

    // public doBannedUser(chatMessage: ChatMsg): void {
    //     if (!!chatMessage && !!chatMessage.nickname) {
    //           this.bannedUser.emit(chatMessage.nickname);
    //     }
    // }

    public isSelf(nickname: string): boolean {
        return (this.nickname === nickname);
    }
    public isToday(value: StringDateTime | null | undefined): boolean {
        let result: boolean = false;
        if (!!value && value.length > 0) {
            result = DateUtil.compare(StringDateTimeUtil.toDate(value), new Date(Date.now())) == 0;
        }
        return result;
    }

    // public isBannedUserById(nickname: string): boolean {
    //     return this.bannedUserIds.includes(nickname);
    // }

    public doKeydownEnter(event: Event, newMsg: string): void {
        const keyEvent: KeyboardEvent = (event as KeyboardEvent);
        if (!keyEvent.altKey && !keyEvent.shiftKey) {
            this.doSendMessage(newMsg);
        }
        //const keyEvent: KeyboardEvent = (event as KeyboardEvent);
        // const textArea: HTMLTextAreaElement = this.getTextArea();
        //   && !!textArea.value && textArea.value.length > 0
        // if (!keyEvent.altKey && !keyEvent.ctrlKey && !keyEvent.shiftKey && !!newMsg && newMsg.length > 0) {
        // this.doSendMessage(textArea.value);
        // this.doSendMessage(newMsg);
        // this.cleanNewMsg();
        // textArea.value = '';
        //}
        event.preventDefault();
    }

    public doKeydownEscape(): void {
        if (!!this.msgEditing) {
            this.cleanNewMsg();
        }
    }

    // ** **
    /*
    public clickFaceSmilePanel(code: string): void {
        const item: HTMLTextAreaElement = this.getTextArea();
        const start = item.selectionStart;
        const value = item.value;
        item.value = value.substr(0, start) + code + value.substr(start);
        item.selectionStart = start + code.length;
        item.selectionEnd = item.selectionStart;
        this.isShowFaceSmilePanel = false;
        this.messageInput?.focus();
    }
    */
    // ** Private API **

    private getMenuData(member: string, selfName: string): MenuData {
        const value = member == selfName;
        return {
            isEdit: value,
            isRemove: value,
        }
    }
    private setTextareaValue(value: string | null): void {
        this.initValue = value;
        this.formControl.setValue(value);
    }

    /*
    private getTextArea(): HTMLTextAreaElement {
        return document.getElementsByClassName('prc-new-message')[0].getElementsByTagName('textarea')[0];
    }*/
    private scrollToBottom(): void {
        try {
            this.scrollItemContainer.nativeElement.scrollTop = this.scrollItemContainer.nativeElement.scrollHeight;
        } catch (err) { }
    }
    /*
    private hexToUtf8(hex: string): string {
        return decodeURIComponent(
            '%' + ((hex || '').match(/.{1,2}/g) || []).join('%')
        );
    }
    */


    private getChatMsg(nickname: string, len: number): ChatMsg[] {
        const result: ChatMsg[] = [];

        for (let idx = 0; idx < len; idx++) {
            let member = "Teodor_Nickols";
            let d1 = new Date((idx < (len / 2) ? -100000000 : 0) + Date.now());
            let date = d1.toISOString();
            let msg = "text_" + idx + " This function can be used to pass through a successful result while handling an error.";
            if (idx % 3 == 0) {
                member = nickname;
            } else if (idx % 2 == 0) {
                member = "Snegana_Miller";
            }
            // const date1 = date.slice(20, 24) + '_' + date.slice(11, 19) + '_' + date.slice(0, 10);
            result.push({ id: idx, date, member, msg });
            this.wait(1);
        }
        return result;
    }

    wait(ms: number): void {
        const start = Date.now();
        let now = start;
        while (now - start < ms) {
            now = Date.now();
        }
    }
}
