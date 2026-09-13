import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBrowseActions } from './panel-browse-actions';

describe('PanelBrowseActions', () => {
  let component: PanelBrowseActions;
  let fixture: ComponentFixture<PanelBrowseActions>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBrowseActions]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBrowseActions);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
