import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBrowseView } from './panel-browse-view';

describe('PanelBrowseView', () => {
  let component: PanelBrowseView;
  let fixture: ComponentFixture<PanelBrowseView>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBrowseView]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBrowseView);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
