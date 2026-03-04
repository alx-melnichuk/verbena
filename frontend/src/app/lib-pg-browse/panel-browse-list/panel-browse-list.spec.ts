import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBrowseList } from './panel-browse-list';

describe('PanelBrowseList', () => {
  let component: PanelBrowseList;
  let fixture: ComponentFixture<PanelBrowseList>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBrowseList]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBrowseList);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
