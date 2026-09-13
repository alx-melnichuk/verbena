import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBrowseRecord } from './panel-browse-record';

describe('PanelBrowseRecord', () => {
  let component: PanelBrowseRecord;
  let fixture: ComponentFixture<PanelBrowseRecord>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBrowseRecord]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBrowseRecord);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
