import {
  ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, Output, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { FieldDragAndDropDirective } from 'src/app/directives/field-drag-and-drop.directive';

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
export class FieldFileUploadComponent {
  @Input()
  public isDisabled: boolean = false;
  @Input()
  public isReadonly: boolean = false;
  @Input()
  public isMultiple = false;
  @Input()
  public maxFileSize = -1;
  @Input()
  public validFileTypes = '';

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

  constructor(private translate: TranslateService) {
  }

  // ** Public API **

  // on file drop handler
  public doFileDropped(event: FileList): void {
    if (!this.isDisabled && !this.isReadonly) {
      this.prepareFilesList(event);
    }
  }
  // handle file from browsing
  public fileBrowseHandler(target: any): void {
    if (!this.isDisabled && !this.isReadonly) {
      this.prepareFilesList(target.files);
    }
  }
  // Delete file from files list
  public deleteFile(index: number): void {
    this.files.splice(index, 1);
  }
  // Convert Files list to normal array list
  public prepareFilesList(files: FileList): void {
    for (let idx = 0, len = files.length; idx < len; idx++) {
      const itemFile: File = files[idx];
      if (this.checkFile(itemFile, this.validFileTypes, this.maxFileSize)) {
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
  private checkFile(file: File, validFileTypes: string, maxFileSize: number): boolean {
    let result = false;
    const types = validFileTypes.split(','); // ['png', 'jpg', 'jpeg', 'gif'];
    const isExist = types.some((item) => file.type.includes(item));
    if (!!types && !isExist) {
      const msg = this.translate.instant('field-file-upload.upload_images_use_valid_types', { 'validTypes': types });
      alert(msg);
    } else if (maxFileSize > 0 && file.size > maxFileSize) {
      const msg = this.translate.instant('field-file-upload.file_size_must_not_exceed_max', { 'maxFileSize': maxFileSize });
      alert(msg);
    } else {
      result = true;
    }
    return result;
  }
}
