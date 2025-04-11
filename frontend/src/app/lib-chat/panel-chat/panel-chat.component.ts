import {
    afterNextRender, AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, EventEmitter, inject, Injector, Input, OnChanges,
    Output, SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CdkTextareaAutosize } from '@angular/cdk/text-field';
import { CommonModule, KeyValue } from '@angular/common';
import { DateAdapter } from '@angular/material/core';
import { MatInput, MatInputModule } from '@angular/material/input';
import { TranslatePipe } from '@ngx-translate/core';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatButtonModule } from '@angular/material/button';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { StringDateTime } from 'src/app/common/string-date-time';

interface ChatMsg {
    msg: string;
    member: string;
    date: StringDateTime;
    // id: string;
    // userId: string;
    // nickname: string;
    // avatar?: string;
    // event: string;
    // text: string;
}

export const PIPE_DATE_COMPACT = 'MMM dd yyyy';
export const PIPE_TIME_SHORT = 'HH:mm aa';

export const MIN_ROWS = 1;
export const MAX_ROWS = 3;

// <mat-form-field subscriptSizing="dynamic"
// it'll remove the space until an error or hint actually needs to get displayed and only then expands.

@Component({
    selector: 'app-panel-chat',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatInputModule, MatFormFieldModule, TranslatePipe, DateTimeFormatPipe],
    templateUrl: './panel-chat.component.html',
    styleUrl: './panel-chat.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelChatComponent implements OnChanges, AfterViewInit {
    @Input()
    public isBlocked: boolean | null = null;
    @Input()
    public isEditable: boolean | null = null;
    @Input()
    public isMobile: boolean | null = null;
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
    public chatMsgs: ChatMsg[] = [];
    @Input()
    public isStreamOwner: boolean | null = true;
    @Input()
    public isUserBanned: boolean | null = null;
    @Input()
    public bannedUserIds: string[] = [];

    @Output()
    readonly sendMsg: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly removeMsg: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly editMsg: EventEmitter<KeyValue<string, string>> = new EventEmitter();
    //   @Output()
    //   readonly bannedUser: EventEmitter<string> = new EventEmitter();

    @ViewChild('autosize')
    public autosize!: CdkTextareaAutosize;
    // @ViewChild('scrollItem')
    // private scrollItemContainer: ElementRef | undefined;
    // @ViewChild(MatInput)
    // private messageInput: MatInput | undefined;
    @ViewChild('textareaElement')
    public textareaElem!: ElementRef<HTMLTextAreaElement>;

    public modifyMsgId: string | null = null;
    // #public newMessage = '';  // formControl.value
    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ newMsg: this.formControl });

    public maxLen: number = 255;
    public maxRowsVal: number = MAX_ROWS;
    public minRowsVal: number = MIN_ROWS;

    public formatDateCompact = PIPE_DATE_COMPACT;
    public formatTimeShort = PIPE_TIME_SHORT;
    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'medium' };
    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: 'medium', timeStyle: 'short' };

    // public isShowFaceSmilePanel = false;

    // public faceSmileList: string[] = [
    //     ...this.getEmojiPart0(),
    //     ...this.getEmojiPart2(),
    // ];

    private mapHoverPrimary: { [key: string]: boolean } = {};
    private mapHoverSecondary: { [key: string]: boolean } = {};

    private readonly dateAdapter: DateAdapter<Date> = inject(DateAdapter);
    private _injector = inject(Injector);

    constructor() {
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
        // console.log(`!!this.messageInput: ${!!this.messageInput}`);
        // console.log(`!!this.textareaElem: ${!!this.textareaElem}`);
        // console.log(`!!this.autosize: ${!!this.autosize}`);
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['maxRows']) {
            this.maxRowsVal = (!!this.maxRows && this.maxRows > 0 ? this.maxRows : MAX_ROWS);
        }
        if (!!changes['minRows']) {
            this.minRowsVal = (!!this.minRows && this.minRows > 0 ? this.minRows : MIN_ROWS);
        }
        if (!!changes['chatMsgs']) {
            this.scrollToBottom();
            Promise.resolve().then(() =>
                this.scrollToBottom());
        }
        if (!!changes['nickname']) {
            this.chatMsgs = this.getChatMsg(this.nickname || ''); // #
        }
    }

    // ** Public API **

    public trackById(index: number, item: ChatMsg): string {
        return item.date;
    }

    public doSendMessage(newMsg: string): void {
        if (!!newMsg && newMsg.length > 0) {
            if (!!this.modifyMsgId) {
                const keyValue: KeyValue<string, string> = { key: this.modifyMsgId, value: newMsg };
                this.editMsg.emit(keyValue);
                this.modifyMsgId = null;
            } else {
                this.sendMsg.emit(newMsg);
            }
            this.cleanNewMsg();
            // this.newMessage = '';
            this.scrollToBottom();
        }
    }
    /*
    public doRemoveMessage(chatMessage: ChatMsg): void {
        if (!!chatMessage && !!chatMessage.id) {
              this.removeMessage.emit(chatMessage.id);
        }
    }

    public doEditMessage(chatMessage: ChatMsg): void {
        if (!!chatMessage && !!chatMessage.id) {
            this.newMessage = chatMessage.text;
            this.modifyMsgId = chatMessage.id;
              if (!!this.messageInput) {
                this.messageInput.focus();
              }
        }
    }

    public doBannedUser(chatMessage: ChatMsg): void {
        if (!!chatMessage && !!chatMessage.nickname) {
              this.bannedUser.emit(chatMessage.nickname);
        }
    }
    */
    // public isToday(value: StringDateTime): boolean {
    //     const todayStr = moment().clone().format(MOMENT_ISO8601_DATE);
    //     const todayMoment = moment(todayStr, MOMENT_ISO8601_DATE);
    //     const valueMoment = moment(value, MOMENT_ISO8601);
    //     return todayMoment.isBefore(valueMoment);
    // }
    public isToday(value: String | null | undefined): boolean {
        if (!!value && value.length > 0) {

        }
        // const date = this.dateAdapter.format(value, null);
        return false;
    }
    public isSelf(nickname: string): boolean {
        return (this.nickname === nickname);
    }
    /*
    public isBannedUserById(nickname: string): boolean {
        return this.bannedUserIds.includes(nickname);
    }

    public isHover(chatMessageId: string): boolean {
        return (!!this.mapHoverPrimary[chatMessageId] || !!this.mapHoverSecondary[chatMessageId]);
    }*/
    public doMouseEnter(chatMsgId: string, isPrimary: boolean): void {
        /*if (!!chatMsgId) {
            if (isPrimary) {
                this.mapHoverPrimary[chatMsgId] = true;
            } else {
                this.mapHoverSecondary[chatMsgId] = true;
            }
        }*/
    }
    public doMouseLeave(chatMsgId: string, isPrimary: boolean): void {
        /*if (!!chatMsgId) {
            if (isPrimary) {
                delete this.mapHoverPrimary[chatMsgId];
            } else {
                delete this.mapHoverSecondary[chatMsgId];
            }
        }*/
    }

    public doKeydownEnter(event: Event, newMsg: string): void {
        if (!this.isMobile) {
            const keyEvent: KeyboardEvent = (event as KeyboardEvent);
            if (!keyEvent.altKey && !keyEvent.shiftKey) {
                this.doSendMessage(newMsg);
            }
        }
        /*const keyEvent: KeyboardEvent = (event as KeyboardEvent);
        // const textArea: HTMLTextAreaElement = this.getTextArea();
        //   && !!textArea.value && textArea.value.length > 0
        if (!keyEvent.altKey && !keyEvent.ctrlKey && !keyEvent.shiftKey && !!newMsg && newMsg.length > 0) {
            // this.doSendMessage(textArea.value);
            this.doSendMessage(newMsg);
            this.cleanNewMsg();
            // textArea.value = '';
        }*/
        event.preventDefault();
    }

    public doKeydownEscape(): void {
        if (!!this.modifyMsgId) {
            this.modifyMsgId = null;
            this.cleanNewMsg();
        }
    }

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

    public getMessageRows(messageTest: string): string[] {
        return messageTest.split('\n');
    }
    */
    // ** Private API **

    private cleanNewMsg(): void {
        this.formControl.setValue('');
    }

    /*
    private getTextArea(): HTMLTextAreaElement {
        return document.getElementsByClassName('prc-new-message')[0].getElementsByTagName('textarea')[0];
    }*/
    private scrollToBottom(): void {
        /*if (!!this.scrollItemContainer) {
            try {
                this.scrollItemContainer.nativeElement.scrollTop = this.scrollItemContainer.nativeElement.scrollHeight;
            } catch (err) { }
        }*/
    }
    /*
    private hexToUtf8(hex: string): string {
        return decodeURIComponent(
            '%' + ((hex || '').match(/.{1,2}/g) || []).join('%')
        );
    }
    */


    private getChatMsg(nickname: string): ChatMsg[] {
        const result: ChatMsg[] = [];

        for (let idx = 0; idx < 18; idx++) {
            let member = "Teodor_Nickols";
            let d1 = new Date(Date.now());
            let date = d1.toISOString();
            let msg = "text_" + idx + " This function can be used to pass through a successful result while handling an error.";
            if (idx % 3 == 0) {
                member = nickname;
            } else if (idx % 2 == 0) {
                member = "Snegana_Miller";
            }
            // const date1 = date.slice(20, 24) + '_' + date.slice(11, 19) + '_' + date.slice(0, 10);
            result.push({ msg, member, date });
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
