import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, EventEmitter, inject, Input, OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation } from '@angular/core';
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

export const CNT_ROWS = 2;

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
    public cntRows: number | null = CNT_ROWS;
    @Input()
    public isBlocked: boolean | null = null;
    @Input()
    public isEditable: boolean | null = null;
    @Input()
    public locale: string | null = null;
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

    // @ViewChild('scrollItem')
    // private scrollItemContainer: ElementRef | undefined;
    // @ViewChild(MatInput)
    // private messageInput: MatInput | undefined;
    //   @ViewChild('textarea', { static: true })
    //   private textareaItem: ElementRef | undefined;

    public modifyMsgId: string | null = null;
    // #public newMessage = '';  // formControl.value
    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ newMsg: this.formControl });

    public maxLen = 255;
    public minRows = 1;
    public maxRows = 4;
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

    constructor() {
        console.log(`PanelChat()`); // #
    }

    ngAfterViewInit(): void {
        this.scrollToBottom();
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['cntRows'] && (!this.cntRows || this.cntRows < 1)) {
            this.cntRows = CNT_ROWS;
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
        const keyEvent: KeyboardEvent = (event as KeyboardEvent);
        // const textArea: HTMLTextAreaElement = this.getTextArea();
        //   && !!textArea.value && textArea.value.length > 0
        if (!keyEvent.altKey && !keyEvent.ctrlKey && !keyEvent.shiftKey && !!newMsg && newMsg.length > 0) {
            // this.doSendMessage(textArea.value);
            this.doSendMessage(newMsg);
            this.cleanNewMsg();
            // textArea.value = '';
        }
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
            // console.log(` date=${date}`); // #
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
