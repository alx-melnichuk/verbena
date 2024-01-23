import { ChangeDetectionStrategy, Component, EventEmitter, Input, OnInit, Output, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslateModule } from '@ngx-translate/core';

import { FieldDragAndDropDirective } from 'src/app/directives/field-drag-and-drop.directive';

let idx = 0;

@Component({
  selector: 'app-field-file-upload',
  standalone: true,
  imports: [CommonModule, FieldDragAndDropDirective, TranslateModule],
  templateUrl: './field-file-upload.component.html',
  styleUrls: ['./field-file-upload.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class FieldFileUploadComponent implements OnInit {
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

  public files: File[] = [];
  public id = 'fileDropId_' + (++idx);

  constructor() {
    console.log(`FieldFileUploadComponent()`);
  }

  ngOnInit(): void {
  }

  // ** Public API **

  // on file drop handler
  public doFileDropped(event: FileList): void {
    this.prepareFilesList(event);
  }
  // handle file from browsing
  public fileBrowseHandler(target: any): void {
    this.prepareFilesList(target.files);
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
  private checkFile(file: File, validFileTypes: string, maxFileSize: number): boolean {
    let result = false;
    const types = validFileTypes.split(','); // ['png', 'jpg', 'jpeg', 'gif'];
    const isExist = types.some((item) => file.type.includes(item));
    if (!!types && !isExist) {
      alert('Please upload images and use png/jpg/jpeg/gif file formats.');
    } else if (maxFileSize > 0 && file.size > maxFileSize) {
      alert(`The file size must not exceed the maximum ${maxFileSize}.`);
    } else {
      result = true;
    }
    return result;
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
}
