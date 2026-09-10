# PDFForge

PDFForge is a personal desktop application for working with PDF files directly
on your computer. Documents and processing stay local: no document is uploaded
to the Internet, PDF tools work offline, and no history of processed PDFs is
kept after the application closes. Internet access is used only when you
explicitly request an application update check or download. No telemetry is
collected.

## Download and run

Download the file for your system from the
[latest release](https://github.com/zemoa/pdfforge/releases/latest):

- **Linux x86_64:** download `PDFForge-<version>-linux-x86_64.AppImage`, make it
  executable in your file manager or with `chmod +x <downloaded-file>.AppImage`,
  then run it. Your system may need FUSE to mount the AppImage.
- **Windows x86_64:** download and run
  `PDFForge-<version>-windows-x86_64.exe`. No installer is needed; the executable
  extracts its bundled PDF runtime into your local application-data folder.
  Microsoft WebView2 must be available on the system.

The PDF engine is included on both platforms. The accompanying `.sig` and
`.sha256` files are used to verify in-application updates.

## Screenshots

![PDFForge home screen, offering its Merge, Split, and Redact tools](docs/images/welcome.png)

## Features

### Merge PDFs

- Add at least two PDFs from your folders or by drag and drop, including the
  same file more than once when needed.
- Add a whole folder to include the PDFs directly inside it, initially sorted
  alphabetically; subfolders are not scanned.
- Freely reorder documents by dragging their central thumbnails or using the
  sidebar's up/down arrows, then choose the name and folder for the final PDF.
- Review the summary before confirming. The generated PDF opens when processing
  is complete; source files are never changed.

Pages retain their size, orientation, and content. Before confirmation,
PDFForge warns you about interactive elements it cannot guarantee to preserve.

![Empty Merge workspace, with document and output panels](docs/images/merge.png)

### Split a PDF

- Browse a document's page thumbnails.
- Create one PDF per page, extract selected pages, or compose several
  non-overlapping page groups.
- Choose a base name and folder for the generated files.

When splitting creates several PDFs, PDFForge opens only their containing
folder. Each result retains its pages' size and orientation, without modifying
the source document.

![Empty Split workspace, with document sidebar and page area](docs/images/split.png)

### Permanently redact information

- Select one or more words, or draw rectangles over sensitive areas — text,
  images, icons, or other content.
- Enlarge and zoom into pages to prepare your selections, then review, edit, or
  remove them before confirming.
- Generate a new PDF where selected areas are black and redacted information
  cannot be recovered.

By default, the result is saved next to the source document as
`<document-name>-masked.pdf`. Both its name and folder can be changed before
confirmation.

To make redaction permanent, every output page is rendered as an image at
300 ppi. The result keeps its page dimensions and visual orientation, but text
is no longer selectable or searchable, and links, forms, bookmarks and source
metadata are not retained. Keep the original if you need those features.

![Empty Redact workspace, with source and selection panels](docs/images/redact.png)

## Safe, predictable workflow

- Before any PDF is created, a summary describes the expected result and asks
  for confirmation.
- Progress is shown during processing, and an operation can be cancelled.
  Partial output files are then removed.
- If an output name already exists, PDFForge automatically appends a number —
  for example, `document-1.pdf` — without overwriting the existing file.
- Choose an existing destination folder or paste its path. On Windows, you can
  also copy a folder in Explorer and paste it into the destination field.
- Password-protected, unreadable, or inaccessible PDFs never trigger a password
  prompt: you can choose to skip them or stop the preparation.
- The application warns before closing during an active PDF operation. After
  success or cancellation, the workspace is cleared for the next task.
- English and French are available, with the system language selected at
  startup and English as the fallback. Switch languages and choose a light,
  dark or system theme from the home screen.
- Documents can range from a few pages to several hundred pages.

## Application updates

On the home screen, click the **PDFForge version** link beside the preferences
to open **About and updates**, then choose **Check for updates**. PDFForge never
checks or downloads automatically. It lists the release notes for all stable
versions newer than the installed version, newest first.

Choose **Download and install** to install the latest stable version. PDFForge
verifies its cryptographic signature and checksum before replacing the
application and restarting. If the application folder is not writable, the
verified download is saved in Downloads for manual replacement.

One previous version is kept after an update and can be restored from the same
dialog. Finish your PDF work before updating or restoring: restarting can
discard a preparation that has not been processed yet.

## Availability

PDFForge is a portable application for Linux x86_64 and Windows x86_64. It is
licensed under [AGPL-3.0-or-later](LICENSE), copyright Zemoa.

## Project documentation

The [functional specification](specs/SFG.md) describes the validated scope. To
contribute to the project, see the [architecture decisions](ARCHITECTURE.md) and
the [development guide](DEVELOPMENT.md).
