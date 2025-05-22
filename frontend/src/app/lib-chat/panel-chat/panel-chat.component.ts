import {
    afterNextRender, AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, EventEmitter, HostListener, inject, Injector, Input,
    OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CdkTextareaAutosize } from '@angular/cdk/text-field';
import { CommonModule, KeyValue } from '@angular/common';
import { FormControl, FormGroup, ReactiveFormsModule, ValidationErrors } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatMenuModule } from '@angular/material/menu';
import { TranslatePipe } from '@ngx-translate/core';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { debounceFn } from 'src/app/common/debounce';
import { StringDateTime } from 'src/app/common/string-date-time';
import { DateUtil } from 'src/app/utils/date.utils';
import { ReplaceWithZeroUtil } from 'src/app/utils/replace-with-zero.util';
import { StringDateTimeUtil } from 'src/app/utils/string-date-time.util';

import { ChatMessageDto } from '../chat-message-api.interface';
import { ValidatorUtils } from 'src/app/utils/validator.utils';

interface MenuData {
    isEdit: boolean;
    isRemove: boolean;
}

export const TITLE = 'message';
export const MESSAGE_MAX_ROWS = 3;
export const MESSAGE_MIN_ROWS = 1;
export const MESSAGE_MAX_LENGTH = 255;
export const MESSAGE_MIN_LENGTH = 0;
export const DEBOUNCE_DELAY = 50;
export const MIN_SCROLL_VALUE = 30; // 20;

type ObjChatMsg = { [key: number]: ChatMessageDto };
type MenuDataMap = Map<number, MenuData>;

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
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelChatComponent implements OnChanges, AfterViewInit {
    @Input()
    public chatMsgs: ChatMessageDto[] = [];
    @Input()
    public isEditable: boolean | null = null;
    @Input()
    public locale: string | null = null;
    @Input()
    public maxLen: number | null = null;
    @Input()
    public minLen: number | null = null;
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
    @Output()
    readonly queryChatMsgs: EventEmitter<{ isSortDes: boolean, borderById: number }> = new EventEmitter();

    @ViewChild('autosize')
    public autosize!: CdkTextareaAutosize;
    @ViewChild('scrollItem')
    private scrollItemElem!: ElementRef<HTMLElement>;
    @ViewChild('textareaElement')
    public textareaElem!: ElementRef<HTMLTextAreaElement>;

    public chatMsgList: ChatMessageDto[] = [];
    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ newMsg: this.formControl });
    public maxLenVal: number = MESSAGE_MAX_LENGTH;
    public minLenVal: number = MESSAGE_MIN_LENGTH;
    public maxRowsVal: number = MESSAGE_MAX_ROWS;
    public minRowsVal: number = MESSAGE_MIN_ROWS;
    public msgMarked: ChatMessageDto | null = null;
    public msgEditing: ChatMessageDto | null = null;
    public initValue: string | null = null;

    readonly dbncScrollItem = debounceFn(() => this.doScrollItem(), DEBOUNCE_DELAY);
    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'medium' };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: 'short' };
    readonly menuDataMap: MenuDataMap = new Map();
    readonly objChatMsg: ObjChatMsg = {};

    private lastScrollTop: number = 0;
    private smallestId: number | null = null;
    private largestId: number | null = null;

    private readonly _injector = inject(Injector);

    constructor() {
        console.log(`PanelChat();`); // #
        this.prepareFormGroup(this.maxLenVal, this.minLenVal);
    }

    triggerResize() {
        // Wait for content to render, then trigger textarea resize.
        afterNextRender(
            () => { this.autosize.resizeToFitContent(true); },
            { injector: this._injector },
        );
    }
    ngAfterViewInit(): void {
        // this.demoSrollTop(this.scrollItemElem.nativeElement);
        // Promise.resolve().then(() => this.scrollToBottom(this.scrollItemElem.nativeElement));
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['chatMsgs']) {
            console.log(`PanelChat.OnChange('chatMsgs') 1 chatMsgs.length: ${this.chatMsgs.length}`);
            this.chatMsgList = this.loadChatMsgs(this.objChatMsg, this.chatMsgs, this.menuDataMap, this.nickname || '');
            this.smallestId = this.chatMsgList[0].id;
            this.largestId = this.chatMsgList[this.chatMsgList.length - 1].id;
            Promise.resolve().then(() => this.scrollToBottom(this.scrollItemElem.nativeElement));
        }
        if (!!changes['isEditable']) {
            Promise.resolve().then(() => this.scrollToBottom(this.scrollItemElem.nativeElement));
        }
        if (!!changes['maxLen'] || !!changes['minLen']) {
            this.maxLenVal = (!!this.maxLen && this.maxLen > 0 ? this.maxLen : MESSAGE_MAX_LENGTH);
            this.minLenVal = (!!this.minLen && this.minLen > 0 ? this.minLen : MESSAGE_MIN_LENGTH);
            this.prepareFormGroup(this.maxLenVal, this.minLenVal);
        }
        if (!!changes['maxRows']) {
            this.maxRowsVal = (!!this.maxRows && this.maxRows > 0 ? this.maxRows : MESSAGE_MAX_ROWS);
        }
        if (!!changes['minRows']) {
            this.minRowsVal = (!!this.minRows && this.minRows > 0 ? this.minRows : MESSAGE_MIN_ROWS);
        }
    }

    // ** Public API **

    public trackById(index: number, item: ChatMessageDto): number {
        return item.id;
    }
    public memberWithZero(value: string): string {
        return ReplaceWithZeroUtil.replace(value);
    }
    public doScrollItem(): void {
        const elem = this.scrollItemElem.nativeElement;
        const isMoveUp = this.lastScrollTop > elem.scrollTop;
        this.lastScrollTop = elem.scrollTop;
        console.log(`PanelChat.doScrollItem() elem.scrollTop: ${elem.scrollTop}, this.deltaScroll(elem): ${this.deltaScroll(elem)}`);
        if (isMoveUp) {
            if (this.deltaScroll(elem) < MIN_SCROLL_VALUE && this.smallestId != null) {
                console.log(`PanelChat.doScrollItem() emmit`);
                this.queryChatMsgs.emit({ isSortDes: true, borderById: this.smallestId });
            }
        }
    }
    public cleanNewMsg(): void {
        this.setTextareaValue(null);
        this.msgEditing = null;
    }
    public getErrorMsg(errors: ValidationErrors | null): string {
        return ValidatorUtils.getErrorMsg(errors, TITLE);
    }
    public doSendMessage(newMsg: string): void {
        const newMsgVal = (newMsg || '').trim();
        if (this.isEditable && newMsgVal.length > 0) {
            if (!!this.msgEditing && this.msgEditing.id > 0 && !this.msgEditing.isRmv) {
                const keyValue: KeyValue<number, string> = { key: this.msgEditing.id, value: newMsgVal };
                this.editMsg.emit(keyValue);
            } else {
                this.sendMsg.emit(newMsgVal);
            }
            this.cleanNewMsg();
            // this.scrollToBottom();
        }
    }
    public doRemoveMessage(chatMsg: ChatMessageDto | null): void {
        if (this.isEditable && !!chatMsg && !!chatMsg.id && chatMsg.member == this.nickname && !chatMsg.isRmv) {
            const keyValue: KeyValue<number, string> = { key: chatMsg.id, value: chatMsg.msg };
            this.removeMsg.emit(keyValue);
        }
    }
    public doSetValueForEditing(chatMsg: ChatMessageDto | null): void {
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
        // # const keyEvent: KeyboardEvent = (event as KeyboardEvent);
        // # const textArea: HTMLTextAreaElement = this.getTextArea();
        // #   && !!textArea.value && textArea.value.length > 0
        // # if (!keyEvent.altKey && !keyEvent.ctrlKey && !keyEvent.shiftKey && !!newMsg && newMsg.length > 0) {
        // # this.doSendMessage(textArea.value);
        // # this.doSendMessage(newMsg);
        // # this.cleanNewMsg();
        // # textArea.value = '';
        //}
        event.preventDefault();
    }

    public doKeydownEscape(): void {
        if (!!this.msgEditing) {
            this.cleanNewMsg();
        }
    }

    public doClickCheckSelection(event: Event): void {
        const selectionObj = window.getSelection();
        const selection = !!selectionObj ? selectionObj.toString() : null;
        // # console.log(`doClickCheckSelection(); selection:"${selection}"`); // #
        if (!!selection) {
            event.preventDefault();
            event.stopPropagation();
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

    private setTextareaValue(value: string | null): void {
        this.initValue = value;
        this.formControl.setValue(value);
    }
    private prepareFormGroup(maxLen: number, minLen: number): void {
        this.formControl.clearValidators();
        const paramsObj = {
            ...(maxLen > 0 ? { "maxLength": maxLen } : {}),
            ...(minLen > 0 ? { "minLength": minLen } : {}),
        };
        this.formControl.setValidators([...ValidatorUtils.prepare(paramsObj)]);
        this.formControl.updateValueAndValidity();
    }
    private deltaScroll(elem: HTMLElement | null | undefined): number {
        let result: number = 0;
        if (!!elem) {
            const height = elem.scrollHeight - elem.clientHeight;
            console.log(`deltaScroll() scrollTop: ${elem.scrollTop}, scrollHeight: ${elem.scrollHeight}, height: ${height}`);
            result = Math.round(Math.round(elem.scrollTop / height * 1000) / 10);
        }
        return result;
    }
    private scrollItems(mode: number): void {
        const elem = this.scrollItemElem.nativeElement;
        if (mode == -1) {

        } else if (mode == 0) {
            const isMoveUp = this.lastScrollTop > elem.scrollTop;
            this.lastScrollTop = elem.scrollTop;

        } else if (mode == 1) {

        }
    }
    private scrollToBottom(elem: HTMLElement | null | undefined): void {
        if (!!elem) {
            // const d1 = elem.scrollTop;
            // elem.scrollTop = elem.scrollHeight - elem.clientHeight;
            // console.log(`scrollToBottom() scrollTop1: ${d1}, scrollTop2: ${elem.scrollTop}`); // #
        }
    }
    /*
    private hexToUtf8(hex: string): string {
        return decodeURIComponent(
            '%' + ((hex || '').match(/.{1,2}/g) || []).join('%')
        );
    }
    */
    private loadChatMsgs(objChatMsg: ObjChatMsg, chatMsgs: ChatMessageDto[], menuDataMap: MenuDataMap, selfName: string): ChatMessageDto[] {
        for (let idx = 0; idx < chatMsgs.length; idx++) {
            const chatMsg = chatMsgs[idx];
            objChatMsg[chatMsg.id] = chatMsg;
            if (!!selfName && selfName == chatMsg.member && !chatMsg.isRmv) {
                menuDataMap.set(chatMsg.id, { "isEdit": true, "isRemove": true });
                console.log(`loadChatMsgs() menuDataMap.get(${chatMsg.id},`, JSON.stringify(menuDataMap.get(chatMsg.id)), `)`);
            }
            console.log(`loadChatMsgs(${idx})`, JSON.stringify(chatMsg));
        }
        return Object.values(objChatMsg);
    }
}
