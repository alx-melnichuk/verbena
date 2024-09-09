import {
  ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, OnChanges, Output, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { FieldDragAndDropDirective } from 'src/app/directives/field-drag-and-drop.directive';
import { ValidFileTypesUtil } from 'src/app/utils/valid_file_types.util';

let idx = 0;

@Component({
  selector: 'app-field-file-upload',
  exportAs: 'appFieldFileUpload',
  standalone: true,
  imports: [CommonModule, FieldDragAndDropDirective, TranslateModule],
  templateUrl: './field-file-upload.component.html',
  styleUrls: ['./field-file-upload.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class FieldFileUploadComponent implements OnChanges {
  @Input()
  public acceptList: string | null | undefined;
  @Input()
  public isDisabled: boolean = false;
  @Input()
  public isReadonly: boolean = false;
  @Input()
  public isMultiple = false;
  @Input()
  public maxFileSize: number = -1;

  @Output()
  readonly addFile: EventEmitter<File> = new EventEmitter();
  @Output()
  readonly readFile: EventEmitter<string[]> = new EventEmitter();

  @HostBinding('class.is-disabled')
  public get classIsDisabledVal(): boolean {
    return this.isDisabled
  }
  @HostBinding('class.is-non-event')
  public get isNonEvent(): boolean {
    return this.isDisabled || this.isReadonly;
  }

  public files: File[] = [];
  public id = 'fileDropId_' + (++idx);
  public textAcceptList: string | undefined;
  public textMaxFileSize: string | undefined;

  constructor(private translate: TranslateService) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['acceptList']) {
      this.textAcceptList = this.prepareAcceptList(this.acceptList);
    }
    if (!!changes['maxFileSize']) {
      this.textMaxFileSize = this.prepareMaxFileSize(this.maxFileSize);
    }
  }

  // ** Public API **

  // on file drop handler
  public doFileDropped(event: FileList): void {
    if (!this.isDisabled && !this.isReadonly) {
      this.prepareFilesList(event, this.acceptList, this.maxFileSize);
    }
  }
  // handle file from browsing
  public fileBrowseHandler(target: any): void {
    if (!this.isDisabled && !this.isReadonly) {
      this.prepareFilesList(target.files, this.acceptList, this.maxFileSize);
    }
  }
  // Delete file from files list
  public deleteFile(index: number): void {
    this.files.splice(index, 1);
  }
  // Convert Files list to normal array list
  public prepareFilesList(files: FileList, acceptList: string | null | undefined, maxFileSize: number): void {
    for (let idx = 0, len = files.length; idx < len; idx++) {
      const itemFile: File = files[idx];
      if (this.checkFile(itemFile, acceptList || '', maxFileSize)) {
        this.files.push(itemFile);
        this.addFile.emit(itemFile);
        this.readDataFile(itemFile);
      }
    }
  }
  // format bytes
  public formatBytes(bytes: number, decimals: number): string {
    if (bytes === 0) {
      return '0 Bytes';
    }
    const k = 1024;
    const dm = decimals <= 0 ? 0 : decimals || 2;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return '' + parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
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
  private checkFile(file: File, acceptList: string, maxFileSize: number): boolean {
    let result = false;
    const types = acceptList.split(','); // ['png', 'jpg', 'jpeg', 'gif'];
    const isExist = types.some((item) => file.type.includes(item));
    if (!!types && !isExist) {
      const validTypes = ValidFileTypesUtil.get(acceptList);
      const msg = this.translate.instant('field-file-upload.err_upload_images_use_valid_types', { 'validTypes': validTypes });
      alert(msg);
    } else if (maxFileSize > 0 && file.size > maxFileSize) {
      const msg = this.translate.instant('field-file-upload.err_file_size_must_not_exceed_max', { 'maxFileSize': maxFileSize });
      alert(msg);
    } else {
      result = true;
    }
    return result;
  }
  private prepareMaxFileSize(maxFileSize: number | null | undefined): string {
    let result: string = '';
    if (maxFileSize != null && maxFileSize > 0) {
      const maxSizeStr = this.formatBytes(maxFileSize, 1);
      result = this.translate.instant('field-file-upload.label_max_file_size', { 'maxFileSize': maxSizeStr });
    }
    return result;
  }
  private prepareAcceptList(acceptList: string | null | undefined): string {
    let result: string = '';
    const validTypes = ValidFileTypesUtil.get(acceptList);
    if (validTypes.length > 0) {
      result = this.translate.instant('field-file-upload.label_accepted_file_types', { 'validTypes': validTypes });
    }
    return result;
  }
}
