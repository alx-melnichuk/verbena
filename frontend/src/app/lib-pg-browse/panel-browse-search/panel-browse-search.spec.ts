import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBrowseSearch } from './panel-browse-search';

describe('PanelBrowseSearch', () => {
  let component: PanelBrowseSearch;
  let fixture: ComponentFixture<PanelBrowseSearch>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBrowseSearch]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBrowseSearch);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
