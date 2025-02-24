import {
    ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { FieldDragAndDropDirective } from 'src/app/directives/field-drag-and-drop.directive';
import { ValidFileTypesUtil } from 'src/app/utils/valid_file_types.util';
import { FileSizeUtil } from 'src/app/utils/file_size.util';

let idx = 0;

@Component({
    selector: 'app-field-file-upload',
    exportAs: 'appFieldFileUpload',
    standalone: true,
    imports: [CommonModule, FieldDragAndDropDirective, TranslatePipe],
    templateUrl: './field-file-upload.component.html',
    styleUrl: './field-file-upload.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class FieldFileUploadComponent implements OnChanges {
    @Input()
    // ".doc,.docx,.xls,.xlsx"; ".bmp,.gif"; "image/png,image/jpeg"; "audio/*,video/*,image/*";
    public accepts: string | null | undefined; // Define the file types (separated by commas) available for upload.
    @Input()
    public isDisabled: boolean = false;
    @Input()
    public isBorder: boolean = false;
    @Input()
    public isReadonly: boolean = false;
    @Input()
    public isMultiple = false;
    @Input()
    public isShowMsgFileSize: boolean = false;
    @Input()
    public isShowMsgFileTypes: boolean = false;
    @Input()
    public maxFileSize: number = -1;

    @Output()
    readonly addFile: EventEmitter<File> = new EventEmitter();
    @Output()
    readonly readFile: EventEmitter<string[]> = new EventEmitter();

    @HostBinding('class.is-disabled')
    public get classIsDisabledVal(): boolean {
        return this.isDisabled;
    }
    @HostBinding('class.is-border')
    public get classIsBorderVal(): boolean {
        return this.isBorder;
    }
    @HostBinding('class.is-non-event')
    public get isNonEvent(): boolean {
        return this.isDisabled || this.isReadonly;
    }

    public files: File[] = [];
    public id = 'fileDropId_' + (++idx);
    public availableFileTypes: string = '';
    public availableMaxFileSize: string = '';

    constructor(private translate: TranslateService) {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['accepts']) {
            this.availableFileTypes = ValidFileTypesUtil.text(this.accepts).join(', ').toUpperCase();
        }
        if (!!changes['maxFileSize']) {
            this.availableMaxFileSize = FileSizeUtil.formatBytes(this.maxFileSize, 1);
        }
    }

    // ** Public API **

    public getFileList(target: any): FileList {
        return target.files;
    }
    // on file drop handler => fileHandler($event, accepts, maxFileSize) 
    // handle file from browsing => fileHandler(target.files, accepts, maxFileSize)
    public fileHandler(files: FileList, accepts: string | null | undefined, maxFileSize: number): void {
        if (this.isDisabled || this.isReadonly) {
            return;
        }
        const acceptsSort: string = ValidFileTypesUtil.sorting(accepts || '').join(',');
        const maxFileSizeShort = FileSizeUtil.formatBytes(maxFileSize, 1);

        for (let idx = 0, len = files.length; idx < len; idx++) {
            const file: File = files[idx];
            let msg = '';
            if (!msg && !ValidFileTypesUtil.checkFileByAccept(file.name, file.type, acceptsSort)) {
                const validTypes = ValidFileTypesUtil.text(acceptsSort).join(', ').toUpperCase();
                msg = this.translate.instant('field-file-upload.err_upload_images_use_valid_types', { 'validTypes': validTypes });
            }
            if (!msg && maxFileSize > 0 && file.size > maxFileSize) {
                msg = this.translate.instant('field-file-upload.err_file_size_must_not_exceed_max', { maxFileSize, maxFileSizeShort });
            }
            if (!!msg) {
                alert(msg);
                continue;
            }
            this.files.push(file);
            this.addFile.emit(file);
            this.readDataFile(file);
        }
    }
    // Delete file from files list
    public deleteFile(index: number): void {
        this.files.splice(index, 1);
    }

    // ** Private API **

    private readDataFile(file: File): void {
        const reader = new FileReader();
        reader.onload = (loadEvent) => {
            const result = (loadEvent.target as any).result;
            this.readFile.emit([file.name, result]);
        };
        reader.readAsDataURL(file); // convert to base64 string and render as a preview
    }
}
